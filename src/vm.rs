use crate::core_types::{Value, Address, Memory, MEMORY_SIZE};
use crate::ast::{OpCode, Stmt, Program};

#[derive(Debug, Clone)]
pub struct VmState {
    pub stack: Vec<Value>,
    pub present: Memory,
    pub anamnesis: Memory,
    pub pc: usize, // Program Counter (conceptual, since we are mostly AST walking)
    pub status: EpochStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EpochStatus {
    Running,
    Finished,
    Paradox,
    Error(String),
}

pub struct Executor {
    // Configuration?
}

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    /// Run a single epoch.
    /// Returns the final state of the Present memory and the Status.
    pub fn run_epoch(&self, program: &Program, anamnesis: &Memory) -> (Memory, EpochStatus) {
        let mut state = VmState {
            stack: Vec::new(),
            present: Memory::default(), // P_initial is empty/zero? Spec says "Constructed during execution".
            anamnesis: anamnesis.clone(),
            pc: 0,
            status: EpochStatus::Running,
        };

        match self.execute_block(&program.body, &mut state) {
            Ok(_) => (state.present, EpochStatus::Finished),
            Err(e) => (state.present, EpochStatus::Error(e)),
        }
    }
    
    // Execute a sequence of statements
    fn execute_block(&self, stmts: &[Stmt], state: &mut VmState) -> Result<(), String> {
        for stmt in stmts {
             if let EpochStatus::Paradox = state.status {
                 return Ok(());
             }
             self.step(stmt, state)?;
        }
        Ok(())
    }

    fn step(&self, stmt: &Stmt, state: &mut VmState) -> Result<(), String> {
        match stmt {
            Stmt::Op(op) => self.execute_op(*op, state),
            Stmt::Push(v) => {
                state.stack.push(v.clone());
                Ok(())
            }
            Stmt::PushAddr(a) => {
                state.stack.push(Value::new(a.0 as u64));
                Ok(())
            }
            Stmt::Block(stmts) => self.execute_block(stmts, state),
            Stmt::If { then_branch, else_branch } => {
                let cond = state.stack.pop().ok_or("Stack underflow in IF condition")?;
                if cond.val != 0 {
                    self.execute_block(then_branch, state)
                } else if let Some(else_stmts) = else_branch {
                    self.execute_block(else_stmts, state)
                } else {
                    Ok(())
                }
            }
            Stmt::While { cond, body } => {
                loop {
                    // Evaluate condition
                    self.execute_block(cond, state)?;
                    let res = state.stack.pop().ok_or("Stack underflow in WHILE condition")?;
                    if res.val == 0 {
                        break;
                    }
                    self.execute_block(body, state)?;
                    
                    // Safety break for infinite loops in this naive implementation?
                    // Spec says: "Epoch execution as deterministic state transformation".
                    // "Bounded iteration" applies to Fixed Point, not necessarily inner loops, but §5 implies determinism.
                    // We should probably have a gas limit for robustness.
                }
                Ok(())
            }
        }
    }

    fn execute_op(&self, op: OpCode, state: &mut VmState) -> Result<(), String> {
        match op {
            OpCode::Nop => {},
            OpCode::Pop => { state.stack.pop(); },
            OpCode::Dup => {
                let val = state.stack.last().ok_or("Stack underflow")?.clone();
                state.stack.push(val);
            },
            OpCode::Swap => {
                let a = state.stack.pop().ok_or("Stack underflow")?;
                let b = state.stack.pop().ok_or("Stack underflow")?;
                state.stack.push(a);
                state.stack.push(b);
            },
            OpCode::Over => {
                if state.stack.len() < 2 { return Err("Stack underflow".to_string()); }
                let val = state.stack[state.stack.len() - 2].clone();
                state.stack.push(val);
            },
            
            // Arithmetic (Traits implemented on Value handle provenance merge)
            OpCode::Add => {
                let b = state.stack.pop().ok_or("Stack underflow")?;
                let a = state.stack.pop().ok_or("Stack underflow")?;
                state.stack.push(a + b);
            },
            OpCode::Sub => {
                let b = state.stack.pop().ok_or("Stack underflow")?;
                let a = state.stack.pop().ok_or("Stack underflow")?;
                state.stack.push(a - b);
            },
            OpCode::Mul => {
                let b = state.stack.pop().ok_or("Stack underflow")?;
                let a = state.stack.pop().ok_or("Stack underflow")?;
                state.stack.push(a * b);
            },
            OpCode::Div => {
                let b = state.stack.pop().ok_or("Stack underflow")?;
                let a = state.stack.pop().ok_or("Stack underflow")?;
                // Div by zero handled inside Value::div
                state.stack.push(a / b);
            },
            OpCode::Mod => {
                let b = state.stack.pop().ok_or("Stack underflow")?;
                let a = state.stack.pop().ok_or("Stack underflow")?;
                state.stack.push(a % b);
            },
            
            // Logic - bitwise ops not traits in core_types yet?
            OpCode::Not => {
                let a = state.stack.pop().ok_or("Stack underflow")?;
                state.stack.push(Value { val: !a.val, prov: a.prov }); // Provenance passes through
            },
            OpCode::And => {
                let b = state.stack.pop().ok_or("Stack underflow")?;
                let a = state.stack.pop().ok_or("Stack underflow")?;
                state.stack.push(Value { 
                    val: a.val & b.val, 
                    prov: a.prov.merge(&b.prov) 
                });
            },
            OpCode::Or => {
                let b = state.stack.pop().ok_or("Stack underflow")?;
                let a = state.stack.pop().ok_or("Stack underflow")?;
                state.stack.push(Value { 
                    val: a.val | b.val, 
                    prov: a.prov.merge(&b.prov) 
                });
            },
            OpCode::Xor => {
                let b = state.stack.pop().ok_or("Stack underflow")?;
                let a = state.stack.pop().ok_or("Stack underflow")?;
                state.stack.push(Value { 
                    val: a.val ^ b.val, 
                    prov: a.prov.merge(&b.prov) 
                });
            },
            
            // Compare results are boolean (1/0).
            // Provenance of the result depends on inputs.
            OpCode::Eq => {
                let b = state.stack.pop().ok_or("Stack underflow")?;
                let a = state.stack.pop().ok_or("Stack underflow")?;
                state.stack.push(Value { 
                    val: if a.val == b.val { 1 } else { 0 },
                    prov: a.prov.merge(&b.prov)
                });
            },
            OpCode::Neq => {
                let b = state.stack.pop().ok_or("Stack underflow")?;
                let a = state.stack.pop().ok_or("Stack underflow")?;
                state.stack.push(Value { 
                    val: if a.val != b.val { 1 } else { 0 },
                    prov: a.prov.merge(&b.prov)
                });
            },
            OpCode::Gt => {
                let b = state.stack.pop().ok_or("Stack underflow")?;
                let a = state.stack.pop().ok_or("Stack underflow")?;
                state.stack.push(Value { 
                    val: if a.val > b.val { 1 } else { 0 },
                    prov: a.prov.merge(&b.prov)
                });
            },
            OpCode::Lt => {
                let b = state.stack.pop().ok_or("Stack underflow")?;
                let a = state.stack.pop().ok_or("Stack underflow")?;
                state.stack.push(Value { 
                    val: if a.val < b.val { 1 } else { 0 },
                    prov: a.prov.merge(&b.prov)
                });
            },

            // TEMPORAL
            OpCode::Oracle => {
                let addr_val = state.stack.pop().ok_or("Stack underflow")?;
                let addr = Address(addr_val.val as u16);
                
                // Read from Anamnesis
                let mut val = state.anamnesis.read(addr);
                
                // INJECT PROVENANCE!
                // The value read depends on this address in Anamnesis.
                let oracle_prov = crate::provenance::Provenance::single(addr);
                
                // Merge with address calculation provenance (if we calculated the address dynamically)
                val.prov = val.prov.merge(&oracle_prov).merge(&addr_val.prov);
                
                state.stack.push(val);
            },
            OpCode::Prophecy => {
                let addr_val = state.stack.pop().ok_or("Stack underflow")?;
                let val = state.stack.pop().ok_or("Stack underflow")?;
                let addr = Address(addr_val.val as u16);
                state.present.write(addr, val);
                // The written value carries its provenance into the Present.
            },
            OpCode::Paradox => {
                state.status = EpochStatus::Paradox;
            },
            
            OpCode::Input => {
                state.stack.push(Value::new(42));
            },
            OpCode::Output => {
                let v = state.stack.pop().ok_or("Stack underflow")?;
                println!("OUTPUT: {:?} (Deps: {:?})", v.val, v.prov);
            },
        }
        Ok(())
    }
}
