use crate::ast::{Program, Stmt, OpCode};
use crate::core_types::{Address, MEMORY_SIZE};
use std::fmt::Write;

/// Compiles a Program into SMT-LIB2 format to solve A = F(A).
/// 
/// We model Memory as an Array (Int, Int) or BitVectors.
/// Since Ourochronos is 64-bit, we use (_ BitVec 64).
/// Address is 16-bit, so (_ BitVec 16) for indices.
/// 
/// The constraint is:
/// (assert (= FinalPresent InitialAnamnesis))
pub struct SmtEncoder {
    pub output: String,
    pub var_counter: usize,
}

impl SmtEncoder {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            var_counter: 0,
        }
    }

    pub fn encode(&mut self, program: &Program) -> String {
        self.output.clear();
        writeln!(self.output, "(set-logic QF_ABV)").unwrap(); // Quantifier-Free Arrays & BitVectors
        writeln!(self.output, "(set-option :produce-models true)").unwrap();
        
        // Define Anamnesis as an array
        writeln!(self.output, "(declare-const anamnesis (Array (_ BitVec 16) (_ BitVec 64)))").unwrap();
        
        // Init Present as an array (usually initialized to 0)
        writeln!(self.output, "(declare-const present_init (Array (_ BitVec 16) (_ BitVec 64)))").unwrap();
        // Const array 0 (requires a quantifier or specific solver support, or just assuming default)
        // For QF_ABV, we can't easily say "const array".
        // Instead, we track writes. 
        // Logic: "present" is state.
        // We will simulate the execution symbolically.
        
        // State: Stack, Present (as symbolic array term)
        // Stack elements are symbolic bitvector terms.
        
        let mut stack: Vec<String> = Vec::new();
        let mut present_term = "present_init".to_string();
        
        // Assume present_init is all zeros?
        // (assert (= present_init ((as const (Array (_ BitVec 16) (_ BitVec 64))) (_ bv0 64))))
        writeln!(self.output, "(assert (= present_init ((as const (Array (_ BitVec 16) (_ BitVec 64))) (_ bv0 64))))").unwrap();

        self.symbolic_exec(&program.body, &mut stack, &mut present_term);
        
        // Final Constraint: Present == Anamnesis
        writeln!(self.output, "(assert (= {} anamnesis))", present_term).unwrap();
        
        writeln!(self.output, "(check-sat)").unwrap();
        writeln!(self.output, "(get-model)").unwrap();
        
        self.output.clone()
    }

    fn symbolic_exec(&mut self, stmts: &[Stmt], stack: &mut Vec<String>, present: &mut String) {
        for stmt in stmts {
            match stmt {
                Stmt::Op(op) => self.encode_op(*op, stack, present),
                Stmt::Push(v) => {
                    let term = format!("(_ bv{} 64)", v.val);
                    stack.push(term);
                },
                Stmt::PushAddr(a) => {
                    // Push constant as u64
                    let term = format!("(_ bv{} 64)", a.0);
                    stack.push(term);
                },
                Stmt::Block(inner) => self.symbolic_exec(inner, stack, present),
                Stmt::If { .. } | Stmt::While { .. } => {
                    // Control flow in SMT is hard. We have to unroll or use ITE (If-Then-Else).
                    // For now, let's assume flat code or simple ITE.
                    // THIS IS A GAP. Full structural translation requires path conditions.
                    // Fallback: Just emit a comment.
                    writeln!(self.output, "; Unsupported control flow").unwrap();
                }
            }
        }
    }

    fn encode_op(&mut self, op: OpCode, stack: &mut Vec<String>, present: &mut String) {
        match op {
            OpCode::Add => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(format!("(bvadd {} {})", a, b));
            },
            OpCode::Sub => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(format!("(bvsub {} {})", a, b));
            },
            OpCode::Mul => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(format!("(bvmul {} {})", a, b));
            },
            OpCode::Div => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(format!("(bvudiv {} {})", a, b));
            },
            OpCode::Mod => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(format!("(bvurem {} {})", a, b));
            },
            
            OpCode::And => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(format!("(bvand {} {})", a, b));
            },
            OpCode::Or => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(format!("(bvor {} {})", a, b));
            },
            OpCode::Xor => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(format!("(bvxor {} {})", a, b));
            },
            OpCode::Not => {
                let a = stack.pop().unwrap();
                stack.push(format!("(bvnot {})", a));
            },
            
            // Compare -> Returns 1 or 0
            OpCode::Eq => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                // (ite (= a b) bv1 bv0)
                stack.push(format!("(ite (= {} {}) (_ bv1 64) (_ bv0 64))", a, b));
            },
            // ... other comparisons ...

            OpCode::Oracle => {
                // Pop addr
                let addr_full = stack.pop().unwrap();
                // Extract 16 bits
                let addr_16 = format!("((_ extract 15 0) {})", addr_full);
                // Read from Anamnesis array
                let term = format!("(select anamnesis {})", addr_16);
                stack.push(term);
            },
            
            OpCode::Prophecy => {
                let addr_full = stack.pop().unwrap();
                let val = stack.pop().unwrap();
                let addr_16 = format!("((_ extract 15 0) {})", addr_full);
                // Update 'present' term using store
                // new_present = (store old_present addr val)
                *present = format!("(store {} {} {})", present, addr_16, val);
            },
            
            _ => {}
        }
    }
}
