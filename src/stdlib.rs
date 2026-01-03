//! Standard Library for OUROCHRONOS.
//!
//! Provides common operations and utilities for temporal programming.

use crate::ast::{Stmt, OpCode, Procedure};
use crate::core_types::Value;

/// Standard library module.
pub struct StdLib;

impl StdLib {
    /// Get all standard library procedures.
    pub fn procedures() -> Vec<Procedure> {
        vec![
            Self::math_procedures(),
            Self::stack_procedures(),
            Self::memory_procedures(),
            Self::io_procedures(),
        ].into_iter().flatten().collect()
    }
    
    /// Mathematical procedures.
    fn math_procedures() -> Vec<Procedure> {
        vec![
            // MIN(a b -- min)
            Procedure {
                name: "MIN".to_string(),
                params: vec!["a".to_string(), "b".to_string()],
                returns: 1,
                body: vec![
                    // ( a b -- min )
                    Stmt::Op(OpCode::Over),   // a b a
                    Stmt::Op(OpCode::Over),   // a b a b
                    Stmt::Op(OpCode::Gt),     // a b (a>b)
                    Stmt::If {
                        then_branch: vec![
                            Stmt::Op(OpCode::Swap),
                            Stmt::Op(OpCode::Pop),
                        ],
                        else_branch: Some(vec![Stmt::Op(OpCode::Pop)]),
                    },
                ],
            },
            // MAX(a b -- max)
            Procedure {
                name: "MAX".to_string(),
                params: vec!["a".to_string(), "b".to_string()],
                returns: 1,
                body: vec![
                    Stmt::Op(OpCode::Over),
                    Stmt::Op(OpCode::Over),
                    Stmt::Op(OpCode::Lt),
                    Stmt::If {
                        then_branch: vec![
                            Stmt::Op(OpCode::Swap),
                            Stmt::Op(OpCode::Pop),
                        ],
                        else_branch: Some(vec![Stmt::Op(OpCode::Pop)]),
                    },
                ],
            },
            // SQUARE(n -- n^2)
            Procedure {
                name: "SQUARE".to_string(),
                params: vec!["n".to_string()],
                returns: 1,
                body: vec![
                    Stmt::Op(OpCode::Dup),
                    Stmt::Op(OpCode::Mul),
                ],
            },
        ]
    }
    
    /// Stack manipulation procedures.
    fn stack_procedures() -> Vec<Procedure> {
        vec![
            // NIP(a b -- b)
            Procedure {
                name: "NIP".to_string(),
                params: vec!["a".to_string(), "b".to_string()],
                returns: 1,
                body: vec![
                    Stmt::Op(OpCode::Swap),
                    Stmt::Op(OpCode::Pop),
                ],
            },
            // TUCK(a b -- b a b)
            Procedure {
                name: "TUCK".to_string(),
                params: vec!["a".to_string(), "b".to_string()],
                returns: 3,
                body: vec![
                    Stmt::Op(OpCode::Swap),
                    Stmt::Op(OpCode::Over),
                ],
            },
            // 2DUP(a b -- a b a b)
            Procedure {
                name: "2DUP".to_string(),
                params: vec!["a".to_string(), "b".to_string()],
                returns: 4,
                body: vec![
                    Stmt::Op(OpCode::Over),
                    Stmt::Op(OpCode::Over),
                ],
            },
            // 2DROP(a b -- )
            Procedure {
                name: "2DROP".to_string(),
                params: vec!["a".to_string(), "b".to_string()],
                returns: 0,
                body: vec![
                    Stmt::Op(OpCode::Pop),
                    Stmt::Op(OpCode::Pop),
                ],
            },
        ]
    }
    
    /// Memory utility procedures.
    fn memory_procedures() -> Vec<Procedure> {
        vec![
            // ZERO(addr -- ) Clear a memory cell
            Procedure {
                name: "ZERO".to_string(),
                params: vec!["addr".to_string()],
                returns: 0,
                body: vec![
                    Stmt::Push(Value::new(0)),
                    Stmt::Op(OpCode::Prophecy),
                ],
            },
            // INC_MEM(addr -- ) Increment memory cell
            Procedure {
                name: "INC_MEM".to_string(),
                params: vec!["addr".to_string()],
                returns: 0,
                body: vec![
                    Stmt::Op(OpCode::Dup),
                    Stmt::Op(OpCode::Oracle),
                    Stmt::Push(Value::new(1)),
                    Stmt::Op(OpCode::Add),
                    Stmt::Op(OpCode::Prophecy),
                ],
            },
        ]
    }
    
    /// I/O utility procedures.
    fn io_procedures() -> Vec<Procedure> {
        vec![
            // NEWLINE(-- ) Output a newline
            Procedure {
                name: "NEWLINE".to_string(),
                params: vec![],
                returns: 0,
                body: vec![
                    Stmt::Push(Value::new(10)), // ASCII newline
                    Stmt::Op(OpCode::Output),
                ],
            },
            // SPACE(-- ) Output a space
            Procedure {
                name: "SPACE".to_string(),
                params: vec![],
                returns: 0,
                body: vec![
                    Stmt::Push(Value::new(32)), // ASCII space
                    Stmt::Op(OpCode::Output),
                ],
            },
        ]
    }
    
    /// Get documentation for all procedures.
    pub fn documentation() -> Vec<(&'static str, &'static str)> {
        vec![
            // Math
            ("MIN", "( a b -- min ) Returns minimum"),
            ("MAX", "( a b -- max ) Returns maximum"),
            ("SQUARE", "( n -- n^2 ) Returns square"),
            // Stack
            ("NIP", "( a b -- b ) Removes second"),
            ("TUCK", "( a b -- b a b ) Tucks top under second"),
            ("2DUP", "( a b -- a b a b ) Duplicates pair"),
            ("2DROP", "( a b -- ) Drops pair"),
            // Memory
            ("ZERO", "( addr -- ) Sets cell to 0"),
            ("INC_MEM", "( addr -- ) Increments cell"),
            // I/O
            ("NEWLINE", "( -- ) Outputs newline"),
            ("SPACE", "( -- ) Outputs space"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stdlib_procedures() {
        let procs = StdLib::procedures();
        assert!(!procs.is_empty());
        assert!(procs.iter().any(|p| p.name == "MIN"));
    }
    
    #[test]
    fn test_stdlib_documentation() {
        let docs = StdLib::documentation();
        assert!(docs.len() >= 10);
    }
}
