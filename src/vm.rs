//! Virtual Machine for OUROCHRONOS epoch execution.
//!
//! The VM executes a single epoch: given an anamnesis (read-only "future" memory),
//! it produces a present (read-write "current" memory) and output.
//!
//! The fixed-point search (in timeloop.rs) repeatedly runs epochs until
//! Present = Anamnesis (temporal consistency achieved).

use crate::core_types::{Value, Address, Memory};
use crate::ast::{OpCode, Stmt, Program};
use crate::provenance::Provenance;
use std::io::{self, Write, BufRead};

/// Status of epoch execution.
#[derive(Debug, Clone, PartialEq)]
pub enum EpochStatus {
    /// Epoch is still running.
    Running,
    /// Epoch completed normally (reached end or HALT).
    Finished,
    /// Epoch terminated due to explicit PARADOX instruction.
    Paradox,
    /// Epoch terminated due to runtime error.
    Error(String),
}

/// Complete state of the VM during epoch execution.
#[derive(Debug, Clone)]
pub struct VmState {
    /// The operand stack.
    pub stack: Vec<Value>,
    /// Present memory (being constructed this epoch).
    pub present: Memory,
    /// Anamnesis memory (read-only, from the "future").
    pub anamnesis: Memory,
    /// Output buffer.
    pub output: Vec<Value>,
    /// Execution status.
    pub status: EpochStatus,
    /// Instruction count (for gas limiting).
    pub instructions_executed: u64,
}

impl VmState {
    /// Create a new VM state for an epoch.
    pub fn new(anamnesis: Memory) -> Self {
        Self {
            stack: Vec::new(),
            present: Memory::new(),
            anamnesis,
            output: Vec::new(),
            status: EpochStatus::Running,
            instructions_executed: 0,
        }
    }
}

/// Result of executing a single epoch.
#[derive(Debug)]
pub struct EpochResult {
    /// The final present memory.
    pub present: Memory,
    /// Output produced during the epoch.
    pub output: Vec<Value>,
    /// Terminal status.
    pub status: EpochStatus,
    /// Number of instructions executed.
    pub instructions_executed: u64,
}

/// Configuration for the executor.
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Maximum instructions per epoch (gas limit).
    pub max_instructions: u64,
    /// Whether to print output immediately.
    pub immediate_output: bool,
    /// Input values (simulated input).
    pub input: Vec<u64>,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_instructions: 10_000_000,
            immediate_output: true,
            input: Vec::new(),
        }
    }
}

/// The OUROCHRONOS virtual machine executor.
pub struct Executor {
    pub config: ExecutorConfig,
    input_cursor: usize,
}

impl Executor {
    /// Create a new executor with default configuration.
    pub fn new() -> Self {
        Self {
            config: ExecutorConfig::default(),
            input_cursor: 0,
        }
    }
    
    /// Create an executor with custom configuration.
    pub fn with_config(config: ExecutorConfig) -> Self {
        Self {
            config,
            input_cursor: 0,
        }
    }
    
    /// Run a single epoch of execution.
    pub fn run_epoch(&mut self, program: &Program, anamnesis: &Memory) -> EpochResult {
        let mut state = VmState::new(anamnesis.clone());
        self.input_cursor = 0;
        
        match self.execute_block(&program.body, &mut state) {
            Ok(_) => {}
            Err(e) => {
                state.status = EpochStatus::Error(e);
            }
        }
        
        // If still running at end, consider it finished
        if state.status == EpochStatus::Running {
            state.status = EpochStatus::Finished;
        }
        
        EpochResult {
            present: state.present,
            output: state.output,
            status: state.status,
            instructions_executed: state.instructions_executed,
        }
    }
    
    /// Execute a block of statements.
    fn execute_block(&mut self, stmts: &[Stmt], state: &mut VmState) -> Result<(), String> {
        for stmt in stmts {
            // Check for termination conditions
            match &state.status {
                EpochStatus::Paradox | EpochStatus::Finished | EpochStatus::Error(_) => {
                    return Ok(());
                }
                EpochStatus::Running => {}
            }
            
            // Check gas limit
            if state.instructions_executed >= self.config.max_instructions {
                state.status = EpochStatus::Error("Instruction limit exceeded".to_string());
                return Ok(());
            }
            
            self.execute_stmt(stmt, state)?;
        }
        Ok(())
    }
    
    /// Execute a single statement.
    fn execute_stmt(&mut self, stmt: &Stmt, state: &mut VmState) -> Result<(), String> {
        state.instructions_executed += 1;
        
        match stmt {
            Stmt::Op(op) => self.execute_op(*op, state),
            
            Stmt::Push(v) => {
                state.stack.push(v.clone());
                Ok(())
            }
            
            Stmt::Block(stmts) => self.execute_block(stmts, state),
            
            Stmt::If { then_branch, else_branch } => {
                let cond = self.pop(state)?;
                if cond.to_bool() {
                    self.execute_block(then_branch, state)
                } else if let Some(else_stmts) = else_branch {
                    self.execute_block(else_stmts, state)
                } else {
                    Ok(())
                }
            }
            
            Stmt::While { cond, body } => {
                loop {
                    // Check termination
                    if state.status != EpochStatus::Running {
                        break;
                    }
                    
                    // Evaluate condition
                    self.execute_block(cond, state)?;
                    let result = self.pop(state)?;
                    
                    if !result.to_bool() {
                        break;
                    }
                    
                    // Execute body
                    self.execute_block(body, state)?;
                    
                    // Gas check for infinite loop prevention
                    if state.instructions_executed >= self.config.max_instructions {
                        state.status = EpochStatus::Error("Instruction limit in loop".to_string());
                        break;
                    }
                }
                Ok(())
            }
            
            Stmt::Call { name } => {
                // Note: Procedure lookup happens at runtime via the program
                // For now, this is a placeholder - actual inlining happens in run_epoch
                Err(format!("Procedure call '{}' not inlined - ensure program is preprocessed", name))
            }
            
            Stmt::Match { cases, default } => {
                // Pop the value to match against
                let val = self.pop(state)?.val;
                
                // Find matching case
                let mut matched = false;
                for (pattern, body) in cases {
                    if val == *pattern {
                        for stmt in body {
                            self.execute_stmt(stmt, state)?;
                        }
                        matched = true;
                        break;
                    }
                }
                
                // Execute default if no match
                if !matched {
                    if let Some(default_body) = default {
                        for stmt in default_body {
                            self.execute_stmt(stmt, state)?;
                        }
                    }
                }
                Ok(())
            }
        }
    }
    
    /// Execute a single opcode.
    fn execute_op(&mut self, op: OpCode, state: &mut VmState) -> Result<(), String> {
        match op {
            // ═══════════════════════════════════════════════════════════
            // Stack Manipulation
            // ═══════════════════════════════════════════════════════════
            
            OpCode::Nop => {}
            
            OpCode::Halt => {
                state.status = EpochStatus::Finished;
            }
            
            OpCode::Pop => {
                self.pop(state)?;
            }
            
            OpCode::Dup => {
                let a = self.peek(state)?;
                state.stack.push(a);
            }
            OpCode::Pick => {
                let n_val = self.pop(state)?;
                let n = n_val.val;
                let len = state.stack.len() as u64;
                if n >= len {
                     return Err(format!("Pick out of bounds: index {} but depth {}", n, len));
                }
                let idx = (len - 1 - n) as usize;
                let val = state.stack[idx].clone();
                state.stack.push(val);
            }
            OpCode::Swap => {
                let a = self.pop(state)?;
                let b = self.pop(state)?;
                state.stack.push(a);
                state.stack.push(b);
            }
            
            OpCode::Over => {
                if state.stack.len() < 2 {
                    return Err("Stack underflow: OVER requires 2 elements".to_string());
                }
                let val = state.stack[state.stack.len() - 2].clone();
                state.stack.push(val);
            }
            
            OpCode::Rot => {
                if state.stack.len() < 3 {
                    return Err("Stack underflow: ROT requires 3 elements".to_string());
                }
                let len = state.stack.len();
                let c = state.stack.remove(len - 3);
                state.stack.push(c);
            }
            
            OpCode::Depth => {
                state.stack.push(Value::new(state.stack.len() as u64));
            }
            
            // ═══════════════════════════════════════════════════════════
            // Arithmetic
            // ═══════════════════════════════════════════════════════════
            
            OpCode::Add => {
                let b = self.pop(state)?;
                let a = self.pop(state)?;
                state.stack.push(a + b);
            }
            
            OpCode::Sub => {
                let b = self.pop(state)?;
                let a = self.pop(state)?;
                state.stack.push(a - b);
            }
            
            OpCode::Mul => {
                let b = self.pop(state)?;
                let a = self.pop(state)?;
                state.stack.push(a * b);
            }
            
            OpCode::Div => {
                let b = self.pop(state)?;
                let a = self.pop(state)?;
                state.stack.push(a / b); // Returns 0 for div by 0
            }
            
            OpCode::Mod => {
                let b = self.pop(state)?;
                let a = self.pop(state)?;
                state.stack.push(a % b); // Returns 0 for mod by 0
            }
            
            OpCode::Neg => {
                let a = self.pop(state)?;
                state.stack.push(Value {
                    val: a.val.wrapping_neg(),
                    prov: a.prov,
                });
            }
            
            // ═══════════════════════════════════════════════════════════
            // Bitwise Logic
            // ═══════════════════════════════════════════════════════════
            
            OpCode::Not => {
                let a = self.pop(state)?;
                state.stack.push(!a);
            }
            
            OpCode::And => {
                let b = self.pop(state)?;
                let a = self.pop(state)?;
                state.stack.push(a & b);
            }
            
            OpCode::Or => {
                let b = self.pop(state)?;
                let a = self.pop(state)?;
                state.stack.push(a | b);
            }
            
            OpCode::Xor => {
                let b = self.pop(state)?;
                let a = self.pop(state)?;
                state.stack.push(a ^ b);
            }
            
            OpCode::Shl => {
                let n = self.pop(state)?;
                let a = self.pop(state)?;
                let shift = (n.val % 64) as u32;
                state.stack.push(Value {
                    val: a.val.wrapping_shl(shift),
                    prov: a.prov.merge(&n.prov),
                });
            }
            
            OpCode::Shr => {
                let n = self.pop(state)?;
                let a = self.pop(state)?;
                let shift = (n.val % 64) as u32;
                state.stack.push(Value {
                    val: a.val.wrapping_shr(shift),
                    prov: a.prov.merge(&n.prov),
                });
            }
            
            // ═══════════════════════════════════════════════════════════
            // Comparison
            // ═══════════════════════════════════════════════════════════
            
            OpCode::Eq => {
                let b = self.pop(state)?;
                let a = self.pop(state)?;
                let prov = a.prov.merge(&b.prov);
                state.stack.push(Value::from_bool_with_prov(a.val == b.val, prov));
            }
            
            OpCode::Neq => {
                let b = self.pop(state)?;
                let a = self.pop(state)?;
                let prov = a.prov.merge(&b.prov);
                state.stack.push(Value::from_bool_with_prov(a.val != b.val, prov));
            }
            
            OpCode::Lt => {
                let b = self.pop(state)?;
                let a = self.pop(state)?;
                let prov = a.prov.merge(&b.prov);
                state.stack.push(Value::from_bool_with_prov(a.val < b.val, prov));
            }
            
            OpCode::Gt => {
                let b = self.pop(state)?;
                let a = self.pop(state)?;
                let prov = a.prov.merge(&b.prov);
                state.stack.push(Value::from_bool_with_prov(a.val > b.val, prov));
            }
            
            OpCode::Lte => {
                let b = self.pop(state)?;
                let a = self.pop(state)?;
                let prov = a.prov.merge(&b.prov);
                state.stack.push(Value::from_bool_with_prov(a.val <= b.val, prov));
            }
            
            OpCode::Gte => {
                let b = self.pop(state)?;
                let a = self.pop(state)?;
                let prov = a.prov.merge(&b.prov);
                state.stack.push(Value::from_bool_with_prov(a.val >= b.val, prov));
            }
            
            // ═══════════════════════════════════════════════════════════
            // Temporal Operations
            // ═══════════════════════════════════════════════════════════
            
            OpCode::Oracle => {
                let addr_val = self.pop(state)?;
                let addr = addr_val.val as Address;
                
                // Read from anamnesis
                let mut val = state.anamnesis.read(addr);
                
                // Inject oracle provenance
                let oracle_prov = Provenance::single(addr);
                val.prov = val.prov.merge(&oracle_prov).merge(&addr_val.prov);
                
                state.stack.push(val);
            }
            
            OpCode::Prophecy => {
                let addr_val = self.pop(state)?;
                let val = self.pop(state)?;
                let addr = addr_val.val as Address;
                
                // Write to present
                state.present.write(addr, val);
            }
            
            OpCode::PresentRead => {
                let addr_val = self.pop(state)?;
                let addr = addr_val.val as Address;
                
                // Read from present (current epoch's memory)
                let mut val = state.present.read(addr);
                val.prov = val.prov.merge(&addr_val.prov);
                
                state.stack.push(val);
            }
            
            OpCode::Paradox => {
                state.status = EpochStatus::Paradox;
            }
            
            // ═══════════════════════════════════════════════════════════
            // I/O
            // ═══════════════════════════════════════════════════════════
            
            OpCode::Input => {
                let val = if self.input_cursor < self.config.input.len() {
                    let v = self.config.input[self.input_cursor];
                    self.input_cursor += 1;
                    v
                } else {
                    // Read from stdin as fallback
                    self.read_input_interactive()
                };
                state.stack.push(Value::new(val));
            }
            
            OpCode::Output => {
                let val = self.pop(state)?;
                
                if self.config.immediate_output {
                    println!("OUTPUT: {} [deps: {:?}]", val.val, val.prov);
                }
                
                state.output.push(val);
            }
            
            // Array/Memory Operations
            OpCode::Pack => {
                // Pack n values into contiguous memory at base address
                let n = self.pop(state)?.val as usize;
                let base = self.pop(state)?.val as Address;
                for i in 0..n {
                    let val = self.pop(state)?;
                    state.present.write(base + (n - 1 - i) as Address, val);
                }
            }
            
            OpCode::Unpack => {
                // Unpack n values from contiguous memory at base address
                let n = self.pop(state)?.val as usize;
                let base = self.pop(state)?.val as Address;
                for i in 0..n {
                    let val = state.present.read(base + i as Address);
                    state.stack.push(val);
                }
            }
            
            OpCode::Index => {
                // Read from base + index
                let index = self.pop(state)?.val as Address;
                let base = self.pop(state)?.val as Address;
                let val = state.present.read(base + index);
                state.stack.push(val);
            }
            
            OpCode::Store => {
                // Store value at base + index
                let index = self.pop(state)?.val as Address;
                let base = self.pop(state)?.val as Address;
                let val = self.pop(state)?;
                state.present.write(base + index, val);
            }
        }
        
        Ok(())
    }
    
    /// Pop a value from the stack.
    fn pop(&self, state: &mut VmState) -> Result<Value, String> {
        state.stack.pop().ok_or_else(|| "Stack underflow".to_string())
    }
    
    /// Peek at the top of the stack.
    fn peek(&self, state: &VmState) -> Result<Value, String> {
        state.stack.last().cloned().ok_or_else(|| "Stack underflow".to_string())
    }
    
    /// Read input interactively.
    fn read_input_interactive(&self) -> u64 {
        print!("INPUT> ");
        io::stdout().flush().ok();
        
        let stdin = io::stdin();
        let mut line = String::new();
        
        if stdin.lock().read_line(&mut line).is_ok() {
            line.trim().parse().unwrap_or(0)
        } else {
            0
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    
    #[test]
    fn test_simple_arithmetic() {
        let program = parse("10 20 ADD").unwrap();
        let mut executor = Executor::new();
        let result = executor.run_epoch(&program, &Memory::new());
        
        assert_eq!(result.status, EpochStatus::Finished);
    }
    
    #[test]
    fn test_oracle_prophecy() {
        let program = parse("0 ORACLE 0 PROPHECY").unwrap();
        let mut anamnesis = Memory::new();
        anamnesis.write(0, Value::new(42));
        
        let mut executor = Executor::new();
        let result = executor.run_epoch(&program, &anamnesis);
        
        assert_eq!(result.status, EpochStatus::Finished);
        assert_eq!(result.present.read(0).val, 42);
    }
    
    #[test]
    fn test_halt() {
        let program = parse("10 HALT 20").unwrap();
        let mut executor = Executor::new();
        let result = executor.run_epoch(&program, &Memory::new());
        
        assert_eq!(result.status, EpochStatus::Finished);
    }
    
    #[test]
    fn test_paradox() {
        let program = parse("PARADOX").unwrap();
        let mut executor = Executor::new();
        let result = executor.run_epoch(&program, &Memory::new());
        
        assert_eq!(result.status, EpochStatus::Paradox);
    }
    
    #[test]
    fn test_division_by_zero() {
        let program = parse("10 0 DIV").unwrap();
        let mut executor = Executor::new();
        let result = executor.run_epoch(&program, &Memory::new());
        
        // Should not error; returns 0
        assert_eq!(result.status, EpochStatus::Finished);
    }
}
