//! Abstract Syntax Tree for OUROCHRONOS.
//!
//! OUROCHRONOS uses a stack-based execution model with structured control flow.
//! Programs consist of statements that manipulate a value stack, present memory,
//! and can read from anamnesis (the temporal oracle).

use crate::core_types::Value;

/// All operations available in OUROCHRONOS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    // ═══════════════════════════════════════════════════════════════════
    // Stack Manipulation
    // ═══════════════════════════════════════════════════════════════════
    
    /// No operation.
    Nop,
    
    /// Halt execution of the current epoch.
    /// Stack: ( -- )
    Halt,
    
    /// Pop and discard the top of stack.
    /// Stack: ( a -- )
    Pop,
    
    /// Duplicate the top of stack.
    /// Stack: ( a -- a a )
    Dup,
    
    /// Swap the top two elements.
    /// Stack: ( a b -- b a )
    Swap,
    
    /// Copy the second element to the top.
    /// Stack: ( a b -- a b a )
    Over,
    
    /// Rotate the top three elements.
    /// Stack: ( a b c -- b c a )
    Rot,
    
    /// Push the current stack depth.
    /// Stack: ( -- n )
    Depth,

    /// Pick the nth element from deep in the stack and copy it to top.
    /// 0 PICK is equivalent to DUP. 1 PICK is equivalent to OVER.
    /// Stack: ( n -- v )
    Pick,
    
    // ═══════════════════════════════════════════════════════════════════
    // Arithmetic (modular, wrapping at 2^64)
    // ═══════════════════════════════════════════════════════════════════
    
    /// Addition.
    /// Stack: ( a b -- a+b )
    Add,
    
    /// Subtraction.
    /// Stack: ( a b -- a-b )
    Sub,
    
    /// Multiplication.
    /// Stack: ( a b -- a*b )
    Mul,
    
    /// Division (returns 0 if divisor is 0).
    /// Stack: ( a b -- a/b )
    Div,
    
    /// Modulo (returns 0 if divisor is 0).
    /// Stack: ( a b -- a%b )
    Mod,
    
    /// Negation (two's complement).
    /// Stack: ( a -- -a )
    Neg,
    
    // ═══════════════════════════════════════════════════════════════════
    // Bitwise Logic
    // ═══════════════════════════════════════════════════════════════════
    
    /// Bitwise NOT.
    /// Stack: ( a -- ~a )
    Not,
    
    /// Bitwise AND.
    /// Stack: ( a b -- a&b )
    And,
    
    /// Bitwise OR.
    /// Stack: ( a b -- a|b )
    Or,
    
    /// Bitwise XOR.
    /// Stack: ( a b -- a^b )
    Xor,
    
    /// Left shift.
    /// Stack: ( a n -- a<<n )
    Shl,
    
    /// Right shift (logical).
    /// Stack: ( a n -- a>>n )
    Shr,
    
    // ═══════════════════════════════════════════════════════════════════
    // Comparison (result: 1 if true, 0 if false)
    // ═══════════════════════════════════════════════════════════════════
    
    /// Equal.
    /// Stack: ( a b -- a==b )
    Eq,
    
    /// Not equal.
    /// Stack: ( a b -- a!=b )
    Neq,
    
    /// Less than.
    /// Stack: ( a b -- a<b )
    Lt,
    
    /// Greater than.
    /// Stack: ( a b -- a>b )
    Gt,
    
    /// Less than or equal.
    /// Stack: ( a b -- a<=b )
    Lte,
    
    /// Greater than or equal.
    /// Stack: ( a b -- a>=b )
    Gte,
    
    // ═══════════════════════════════════════════════════════════════════
    // Temporal Operations (the core of OUROCHRONOS)
    // ═══════════════════════════════════════════════════════════════════
    
    /// ORACLE: Read from anamnesis (the future).
    /// Pops an address, pushes the value from Anamnesis[address].
    /// This is the mechanism for receiving information from the future.
    /// Stack: ( addr -- A[addr] )
    Oracle,
    
    /// PROPHECY: Write to present (fulfilling the future).
    /// Pops value and address, writes value to Present[address].
    /// This is the mechanism for sending information to the past.
    /// Stack: ( value addr -- )
    Prophecy,
    
    /// PRESENT_READ: Read from present memory (current epoch).
    /// Stack: ( addr -- P[addr] )
    PresentRead,
    
    /// PARADOX: Signal explicit inconsistency.
    /// Terminates the current epoch as paradoxical.
    /// In fixed-point search, this path is rejected.
    /// Stack: ( -- )
    Paradox,
    
    // ═══════════════════════════════════════════════════════════════════
    // I/O Operations
    // ═══════════════════════════════════════════════════════════════════
    
    /// Read a value from input.
    /// Stack: ( -- value )
    Input,
    
    /// Output a value.
    /// Stack: ( value -- )
    Output,
}

impl OpCode {
    /// Get the name of this opcode as it appears in source code.
    pub fn name(&self) -> &'static str {
        match self {
            OpCode::Nop => "NOP",
            OpCode::Halt => "HALT",
            OpCode::Pop => "POP",
            OpCode::Dup => "DUP",
            OpCode::Swap => "SWAP",
            OpCode::Over => "OVER",
            OpCode::Rot => "ROT",
            OpCode::Depth => "DEPTH",
            OpCode::Pick => "PICK",
            OpCode::Add => "ADD",
            OpCode::Sub => "SUB",
            OpCode::Mul => "MUL",
            OpCode::Div => "DIV",
            OpCode::Mod => "MOD",
            OpCode::Neg => "NEG",
            OpCode::Not => "NOT",
            OpCode::And => "AND",
            OpCode::Or => "OR",
            OpCode::Xor => "XOR",
            OpCode::Shl => "SHL",
            OpCode::Shr => "SHR",
            OpCode::Eq => "EQ",
            OpCode::Neq => "NEQ",
            OpCode::Lt => "LT",
            OpCode::Gt => "GT",
            OpCode::Lte => "LTE",
            OpCode::Gte => "GTE",
            OpCode::Oracle => "ORACLE",
            OpCode::Prophecy => "PROPHECY",
            OpCode::PresentRead => "PRESENT",
            OpCode::Paradox => "PARADOX",
            OpCode::Input => "INPUT",
            OpCode::Output => "OUTPUT",
        }
    }
    
    /// Get the stack effect: (inputs, outputs).
    pub fn stack_effect(&self) -> (usize, usize) {
        match self {
            OpCode::Nop | OpCode::Halt | OpCode::Paradox => (0, 0),
            OpCode::Pop | OpCode::Output => (1, 0),
            OpCode::Dup => (1, 2),
            OpCode::Swap => (2, 2),
            OpCode::Over => (2, 3),

            OpCode::Rot => (3, 3),
            OpCode::Depth | OpCode::Input => (0, 1),
            OpCode::Pick => (1, 1),
            OpCode::Neg | OpCode::Not => (1, 1),
            OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Mod |
            OpCode::And | OpCode::Or | OpCode::Xor | OpCode::Shl | OpCode::Shr |
            OpCode::Eq | OpCode::Neq | OpCode::Lt | OpCode::Gt | OpCode::Lte | OpCode::Gte => (2, 1),
            OpCode::Oracle | OpCode::PresentRead => (1, 1),
            OpCode::Prophecy => (2, 0),
        }
    }
}

/// A statement in the OUROCHRONOS AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Execute a single opcode.
    Op(OpCode),
    
    /// Push a constant value onto the stack.
    Push(Value),
    
    /// Structured IF statement.
    /// Pops the condition from stack; if non-zero, executes then_branch.
    If {
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    
    /// Structured WHILE loop.
    /// Evaluates cond block, pops result; if non-zero, executes body and repeats.
    While {
        cond: Vec<Stmt>,
        body: Vec<Stmt>,
    },
    
    /// A block of statements (scoped grouping).
    Block(Vec<Stmt>),
}

/// A complete OUROCHRONOS program.
#[derive(Debug, Clone)]
pub struct Program {
    pub body: Vec<Stmt>,
}

impl Program {
    /// Create an empty program.
    pub fn new() -> Self {
        Program { body: Vec::new() }
    }
    
    /// Check if the program is trivially consistent (no oracle operations).
    pub fn is_trivially_consistent(&self) -> bool {
        !self.contains_oracle(&self.body)
    }
    
    fn contains_oracle(&self, stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Op(OpCode::Oracle) => return true,
                Stmt::If { then_branch, else_branch } => {
                    if self.contains_oracle(then_branch) {
                        return true;
                    }
                    if let Some(else_stmts) = else_branch {
                        if self.contains_oracle(else_stmts) {
                            return true;
                        }
                    }
                }
                Stmt::While { cond, body } => {
                    if self.contains_oracle(cond) || self.contains_oracle(body) {
                        return true;
                    }
                }
                Stmt::Block(inner) => {
                    if self.contains_oracle(inner) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}
