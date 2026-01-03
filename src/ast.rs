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

    /// Roll the nth element to the top, shifting others down.
    /// 0 ROLL is Nop. 1 ROLL is SWAP.
    /// Stack: ( n -- )
    Roll,

    /// Reverse the top n elements.
    /// Stack: ( n -- )
    Reverse,
    
    // ════ String Operations ════
    
    /// Reverse a string (len-suffixed sequence).
    /// Stack: ( chars... len -- reversed_chars... len )
    StrRev,
    
    /// Concatenate two strings.
    /// Stack: ( c1.. len1 c2.. len2 -- c1..c2.. (len1+len2) )
    StrCat,
    
    /// Split string by delimiter char.
    /// Stack: ( chars... len delim_char -- s1 s2 .. sn count )
    StrSplit,

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

    /// Assertion.
    /// Pops length, chars, then condition. panics if condition is 0.
    /// Stack: ( cond chars... len -- )
    Assert,
    
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
    // Array/Memory Operations
    // ═══════════════════════════════════════════════════════════════════
    
    /// Pack n values into contiguous memory starting at base address.
    /// Stack: ( v1 v2 ... vn base n -- )
    Pack,
    
    /// Unpack n values from contiguous memory at base address.
    /// Stack: ( base n -- v1 v2 ... vn )
    Unpack,
    
    /// Read from memory at computed index: base + index.
    /// Stack: ( base index -- P[base+index] )
    Index,
    
    /// Store to memory at computed index: base + index.
    /// Stack: ( value base index -- )
    Store,
    
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
            OpCode::Assert => "ASSERT",
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
            OpCode::Pack => "PACK",
            OpCode::Unpack => "UNPACK",
            OpCode::Index => "INDEX",
            OpCode::Store => "STORE",
            OpCode::Input => "INPUT",
            OpCode::Output => "OUTPUT",
            OpCode::Roll => "ROLL",
            OpCode::Reverse => "REVERSE",
            OpCode::StrRev => "STR_REV",
            OpCode::StrCat => "STR_CAT",
            OpCode::StrSplit => "STR_SPLIT",
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
            OpCode::Neg => (1, 1),
            OpCode::Assert => (2, 0), // Effectively consumes cond + len (and chars via len)
            OpCode::Not => (1, 1),
            OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Mod |
            OpCode::And | OpCode::Or | OpCode::Xor | OpCode::Shl | OpCode::Shr |
            OpCode::Eq | OpCode::Neq | OpCode::Lt | OpCode::Gt | OpCode::Lte | OpCode::Gte => (2, 1),
            OpCode::Oracle | OpCode::PresentRead => (1, 1),
            OpCode::Prophecy => (2, 0),
            // Array opcodes have variable effects, these are minimums
            OpCode::Pack => (2, 0),    // base, n, (plus n values consumed)
            OpCode::Unpack => (2, 0),  // base, n (produces n values)
            OpCode::Index => (2, 1),   // base, index -> value
            OpCode::Store => (3, 0),   // value, base, index
            OpCode::Roll => (1, 0),    // pops n
            OpCode::Reverse => (1, 0), // pops n
            OpCode::StrRev => (1, 1),  // pops len, pushes len
            OpCode::StrCat => (2, 1),  // pops len1, len2, pushes len_sum
            OpCode::StrSplit => (2, 1), // variable return
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
    
    /// Call a procedure by name.
    /// Stack effect depends on the procedure's parameter and return count.
    Call {
        name: String,
    },
    
    /// Pattern match on top-of-stack value.
    /// Pops value and selects matching case branch or default.
    Match {
        /// List of (pattern, body) pairs.
        cases: Vec<(u64, Vec<Stmt>)>,
        /// Default case if no pattern matches.
        default: Option<Vec<Stmt>>,
    },
    
    /// Temporal scope block.
    /// TEMPORAL <base> <size> { body }
    /// 
    /// Creates an isolated memory region for temporal operations.
    /// ORACLE/PROPHECY within the block are relative to base address.
    /// Changes within the scope are propagated to parent on successful exit.
    /// If the block induces a paradox, changes are discarded.
    TemporalScope {
        /// Base address for the isolated region.
        base: u64,
        /// Size of the isolated region.
        size: u64,
        /// Body of the temporal scope.
        body: Vec<Stmt>,
    },
}

/// Side effect behavior of a procedure.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Procedure is pure (no side effects, deterministic given inputs).
    /// Implies no ORACLE reads and no PROPHECY writes.
    Pure,
    /// Procedure reads from the specific oracle address.
    Reads(u64),
    /// Procedure writes to the specific prophecy address.
    Writes(u64),
}

/// A procedure definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Procedure {
    /// Procedure name.
    pub name: String,
    /// Parameter names (for documentation; all params come from stack).
    pub params: Vec<String>,
    /// Number of return values pushed to stack.
    pub returns: usize,
    /// Effect annotations declared by the user.
    pub effects: Vec<Effect>,
    /// Body of the procedure.
    pub body: Vec<Stmt>,
}

/// A complete OUROCHRONOS program.
#[derive(Debug, Clone)]
pub struct Program {
    /// Procedure definitions.
    pub procedures: Vec<Procedure>,
    /// Main program body.
    pub body: Vec<Stmt>,
}

impl Program {
    /// Create an empty program.
    pub fn new() -> Self {
        Program { 
            procedures: Vec::new(),
            body: Vec::new(),
        }
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
                Stmt::TemporalScope { body, .. } => {
                    if self.contains_oracle(body) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
    
    /// Inline all procedure calls, replacing Stmt::Call with procedure bodies.
    /// Returns a new Program with all calls inlined.
    pub fn inline_procedures(&self) -> Self {
        let proc_map: std::collections::HashMap<String, &Procedure> = 
            self.procedures.iter().map(|p| (p.name.clone(), p)).collect();
        
        Program {
            procedures: Vec::new(), // Procedures are now inlined
            body: self.inline_stmts(&self.body, &proc_map),
        }
    }
    
    fn inline_stmts(&self, stmts: &[Stmt], procs: &std::collections::HashMap<String, &Procedure>) -> Vec<Stmt> {
        stmts.iter().map(|stmt| self.inline_stmt(stmt, procs)).collect()
    }
    
    fn inline_stmt(&self, stmt: &Stmt, procs: &std::collections::HashMap<String, &Procedure>) -> Stmt {
        match stmt {
            Stmt::Call { name } => {
                if let Some(proc) = procs.get(name) {
                    // Inline the procedure body
                    Stmt::Block(self.inline_stmts(&proc.body, procs))
                } else {
                    // Procedure not found - keep as is (will error at runtime)
                    stmt.clone()
                }
            }
            Stmt::If { then_branch, else_branch } => Stmt::If {
                then_branch: self.inline_stmts(then_branch, procs),
                else_branch: else_branch.as_ref().map(|e| self.inline_stmts(e, procs)),
            },
            Stmt::While { cond, body } => Stmt::While {
                cond: self.inline_stmts(cond, procs),
                body: self.inline_stmts(body, procs),
            },
            Stmt::Block(inner) => Stmt::Block(self.inline_stmts(inner, procs)),
            Stmt::TemporalScope { base, size, body } => Stmt::TemporalScope {
                base: *base,
                size: *size,
                body: self.inline_stmts(body, procs),
            },
            other => other.clone(),
        }
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}
