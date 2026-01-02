use crate::core_types::{Value, Address};

/// Operations available in Ourochronos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    // Stack Manipulation
    Nop,
    Pop,
    Dup,
    Swap,
    Over, // Standard stack op: ( a b -- a b a )
    
    // Arithmetic & Logic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Not, // Bitwise Not
    And,
    Or,
    Xor,
    
    // Comparison (pushes 1 if true, 0 if false)
    Eq,
    Neq,
    Lt,
    Gt,
    
    // Temporal Operations
    /// `ORACLE`: Pops address, pushes value from Anamnesis at that address.
    /// Stack: ( addr -- val )
    Oracle,
    
    /// `PROPHECY`: Pops value and address, writes value to Present at that address.
    /// Stack: ( val addr -- )
    Prophecy,
    
    // Explicit Inconsistency
    /// `PARADOX`: Signals that the current path is inconsistent.
    /// In a fixed-point search, this path is invalid.
    Paradox,
    
    // I/O (for examples)
    Input,  // ( -- val )
    Output, // ( val -- )
}

/// Abstract Syntax Tree for Structured Ourochronos.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Execute a single opcode.
    Op(OpCode),
    
    /// Push a constant value onto the stack.
    Push(Value),
    
    /// Push an Address onto the stack (syntax sugar, basically Push(Value)).
    PushAddr(Address),
    
    /// Structured If: condition is popped from stack.
    /// `Val != 0` is true.
    If {
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    
    /// Structured While loop.
    /// Expects condition to be evaluated before the loop or inside?
    /// Standard structured while: `while (cond) { body }`.
    /// BUT for a stack machine, `while` usually checks the stack.
    /// `While` consumes top of stack? 
    /// Better pattern: `Loop { body... }` with a conditional `Break`?
    /// Or Forth-style: `BEGIN ... WHILE ... REPEAT`.
    /// Let's go with a simple `While(condition_block, body_block)`.
    /// `condition_block` executes, top of stack is checked. If true, `body_block` executes.
    While {
        cond: Vec<Stmt>,
        body: Vec<Stmt>,
    },
    
    /// A block of statements (scope).
    Block(Vec<Stmt>),
}

/// A complete Program.
#[derive(Debug, Clone)]
pub struct Program {
    pub body: Vec<Stmt>,
}
