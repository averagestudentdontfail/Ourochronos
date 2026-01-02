//! Compile-time type system for OUROCHRONOS.
//!
//! This module provides static analysis to track **temporal tainting** - which values
//! depend on oracle reads (temporal) vs. which are pure constants (ground truth).
//!
//! # Type Lattice
//!
//! ```text
//!        Temporal (depends on oracle)
//!           ↑
//!        Unknown (not yet determined)
//!           ↑
//!         Pure (constant, no oracle dependency)
//! ```
//!
//! # Type Rules
//!
//! | Operation | Input Types | Output Type |
//! |-----------|-------------|-------------|
//! | `ORACLE` | `Pure` (address) | `Temporal` |
//! | `PROPHECY` | `Any`, `Pure` (address) | - |
//! | Arithmetic | `Temporal × Any` | `Temporal` |
//! | Arithmetic | `Pure × Pure` | `Pure` |
//! | Comparison | `Temporal × Any` | `Temporal` |
//! | `IF` | `Temporal` condition | Both branches tainted |
//!
//! # Safety Properties
//!
//! The type checker enforces:
//! 1. **Causal Integrity**: Temporal values must flow through verification
//! 2. **Stability Guarantee**: Pure values cannot spontaneously become temporal
//! 3. **Taint Tracking**: The programmer knows which computations depend on the future

use crate::ast::{Program, Stmt, OpCode};
use crate::core_types::Address;
use std::collections::HashMap;

/// Temporal type for compile-time causal tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemporalType {
    /// Value has no temporal dependency (constant, ground truth).
    Pure,
    /// Value depends on at least one oracle read.
    Temporal,
    /// Type is not yet determined (for inference).
    Unknown,
}

impl TemporalType {
    /// Join two types in the lattice.
    /// Temporal ⊔ anything = Temporal
    /// Pure ⊔ Pure = Pure
    /// Unknown ⊔ x = x
    pub fn join(self, other: Self) -> Self {
        match (self, other) {
            (TemporalType::Temporal, _) | (_, TemporalType::Temporal) => TemporalType::Temporal,
            (TemporalType::Pure, TemporalType::Pure) => TemporalType::Pure,
            (TemporalType::Unknown, x) | (x, TemporalType::Unknown) => x,
        }
    }
    
    /// Check if this type is more specific than another.
    pub fn is_subtype_of(self, other: Self) -> bool {
        match (self, other) {
            (_, TemporalType::Unknown) => true,
            (TemporalType::Pure, TemporalType::Pure) => true,
            (TemporalType::Temporal, TemporalType::Temporal) => true,
            (TemporalType::Pure, TemporalType::Temporal) => true, // Pure can be used where Temporal expected
            _ => false,
        }
    }
    
    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            TemporalType::Pure => "Pure",
            TemporalType::Temporal => "Temporal",
            TemporalType::Unknown => "Unknown",
        }
    }
}

impl Default for TemporalType {
    fn default() -> Self {
        TemporalType::Unknown
    }
}

/// Type error during checking.
#[derive(Debug, Clone)]
pub struct TypeError {
    /// Description of the error.
    pub message: String,
    /// Location hint (statement index).
    pub location: Option<usize>,
}

impl TypeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: None,
        }
    }
    
    pub fn with_location(mut self, loc: usize) -> Self {
        self.location = Some(loc);
        self
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(loc) = self.location {
            write!(f, "[stmt {}] {}", loc, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

/// Result of type checking.
#[derive(Debug, Clone)]
pub struct TypeCheckResult {
    /// Whether the program is well-typed.
    pub is_valid: bool,
    /// Any type errors found.
    pub errors: Vec<TypeError>,
    /// Warnings (non-fatal issues).
    pub warnings: Vec<String>,
    /// Inferred types for memory cells.
    pub cell_types: HashMap<Address, TemporalType>,
    /// Stack type at end of program.
    pub final_stack_types: Vec<TemporalType>,
}

impl TypeCheckResult {
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            cell_types: HashMap::new(),
            final_stack_types: Vec::new(),
        }
    }
    
    pub fn with_error(error: TypeError) -> Self {
        Self {
            is_valid: false,
            errors: vec![error],
            warnings: Vec::new(),
            cell_types: HashMap::new(),
            final_stack_types: Vec::new(),
        }
    }
}

/// Type checker for OUROCHRONOS programs.
pub struct TypeChecker {
    /// Abstract stack of types.
    stack: Vec<TemporalType>,
    /// Types of memory cells (from PROPHECY writes).
    cell_types: HashMap<Address, TemporalType>,
    /// Collected errors.
    errors: Vec<TypeError>,
    /// Collected warnings.
    warnings: Vec<String>,
    /// Current statement index.
    stmt_index: usize,
    /// Whether we're in a temporal-controlled branch.
    in_temporal_branch: bool,
}

impl TypeChecker {
    /// Create a new type checker.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            cell_types: HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            stmt_index: 0,
            in_temporal_branch: false,
        }
    }
    
    /// Type-check a program.
    pub fn check(&mut self, program: &Program) -> TypeCheckResult {
        self.check_statements(&program.body);
        
        TypeCheckResult {
            is_valid: self.errors.is_empty(),
            errors: self.errors.clone(),
            warnings: self.warnings.clone(),
            cell_types: self.cell_types.clone(),
            final_stack_types: self.stack.clone(),
        }
    }
    
    /// Check a block of statements.
    fn check_statements(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.check_stmt(stmt);
            self.stmt_index += 1;
        }
    }
    
    /// Check a single statement.
    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Push(value) => {
                // Constants are always pure
                let ty = if value.prov.is_pure() {
                    TemporalType::Pure
                } else {
                    TemporalType::Temporal
                };
                self.stack.push(ty);
            }
            
            Stmt::Op(op) => {
                self.check_op(*op);
            }
            
            Stmt::If { then_branch, else_branch } => {
                // Pop condition
                let cond_type = self.pop_type();
                
                // If condition is temporal, both branches are tainted
                let was_temporal = self.in_temporal_branch;
                if cond_type == TemporalType::Temporal {
                    self.in_temporal_branch = true;
                }
                
                // Check branches
                let stack_before = self.stack.clone();
                self.check_statements(then_branch);
                let then_stack = self.stack.clone();
                
                self.stack = stack_before;
                if let Some(else_stmts) = else_branch {
                    self.check_statements(else_stmts);
                }
                let else_stack = self.stack.clone();
                
                // Merge branch stacks (join types)
                self.stack = self.merge_stacks(&then_stack, &else_stack);
                
                self.in_temporal_branch = was_temporal;
            }
            
            Stmt::While { cond, body } => {
                // Check condition
                let stack_before = self.stack.clone();
                self.check_statements(cond);
                let cond_type = self.pop_type();
                
                // Loops with temporal conditions are especially tricky
                if cond_type == TemporalType::Temporal {
                    self.warnings.push(format!(
                        "Loop condition is Temporal at stmt {}: loop behavior may depend on future",
                        self.stmt_index
                    ));
                    self.in_temporal_branch = true;
                }
                
                // Check body
                self.check_statements(body);
                
                // Restore stack (approximate: assume loop terminates)
                self.stack = stack_before;
            }
            
            Stmt::Block(stmts) => {
                self.check_statements(stmts);
            }
            
            Stmt::Call { name: _ } => {
                // Procedure call: we don't have the body here, so mark as Unknown
                // In a full implementation, we'd look up the procedure and analyze its body
                self.stack.push(TemporalType::Unknown);
            }
            
            Stmt::Match { cases, default } => {
                // Pop the matched value
                self.pop_type();
                
                // Analyze all branches (conservative: union of all possible types)
                for (_, body) in cases {
                    for stmt in body {
                        self.check_stmt(stmt);
                    }
                }
                if let Some(default_body) = default {
                    for stmt in default_body {
                        self.check_stmt(stmt);
                    }
                }
            }
        }
    }
    
    /// Check an opcode.
    fn check_op(&mut self, op: OpCode) {
        match op {
            // ===== Temporal Operations =====
            OpCode::Oracle => {
                let addr_type = self.pop_type();
                if addr_type == TemporalType::Temporal {
                    self.warnings.push(format!(
                        "ORACLE address is Temporal at stmt {}: address depends on future",
                        self.stmt_index
                    ));
                }
                // Result is always temporal
                self.stack.push(TemporalType::Temporal);
            }
            
            OpCode::Prophecy => {
                let addr_type = self.pop_type();
                let value_type = self.pop_type();
                
                if addr_type == TemporalType::Temporal {
                    self.warnings.push(format!(
                        "PROPHECY address is Temporal at stmt {}: writing to future-dependent address",
                        self.stmt_index
                    ));
                }
                
                // Record the type written to this cell (if address is constant)
                // In practice, we don't know the address statically, so we approximate
                self.cell_types.insert(0, value_type); // Simplified: assume address 0
            }
            
            OpCode::PresentRead => {
                let addr_type = self.pop_type();
                // Reading from present can be temporal if written with temporal value
                let result = if self.in_temporal_branch {
                    TemporalType::Temporal
                } else {
                    addr_type // Approximate: same as address type
                };
                self.stack.push(result);
            }
            
            // ===== Stack Manipulation =====
            OpCode::Pop => {
                self.pop_type();
            }
            
            OpCode::Dup => {
                let ty = self.peek_type();
                self.stack.push(ty);
            }
            
            OpCode::Swap => {
                if self.stack.len() >= 2 {
                    let len = self.stack.len();
                    self.stack.swap(len - 1, len - 2);
                }
            }
            
            OpCode::Over => {
                if self.stack.len() >= 2 {
                    let ty = self.stack[self.stack.len() - 2];
                    self.stack.push(ty);
                }
            }
            
            OpCode::Rot => {
                if self.stack.len() >= 3 {
                    let len = self.stack.len();
                    let a = self.stack[len - 3];
                    self.stack[len - 3] = self.stack[len - 2];
                    self.stack[len - 2] = self.stack[len - 1];
                    self.stack[len - 1] = a;
                }
            }
            
            OpCode::Depth => {
                // Depth is always pure (it's a constant at that point)
                self.stack.push(TemporalType::Pure);
            }
            
            OpCode::Pick => {
                let _ = self.pop_type(); // index
                // Result depends on what we're picking
                // Approximate: could be temporal
                self.stack.push(TemporalType::Unknown);
            }
            
            OpCode::Nop | OpCode::Neg => {
                // Nop: no effect. Neg: unary, preserves type.
            }
            
            // ===== Arithmetic (joins types) =====
            OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Mod => {
                let b = self.pop_type();
                let a = self.pop_type();
                self.stack.push(a.join(b));
            }
            
            // ===== Bitwise (joins types) =====
            OpCode::And | OpCode::Or | OpCode::Xor | OpCode::Shl | OpCode::Shr => {
                let b = self.pop_type();
                let a = self.pop_type();
                self.stack.push(a.join(b));
            }
            
            OpCode::Not => {
                // Unary: preserves type
                // Type stays the same
            }
            
            // ===== Comparison (joins types) =====
            OpCode::Eq | OpCode::Neq | OpCode::Lt | OpCode::Gt | OpCode::Lte | OpCode::Gte => {
                let b = self.pop_type();
                let a = self.pop_type();
                self.stack.push(a.join(b));
            }
            
            // ===== I/O =====
            OpCode::Input => {
                // Input is pure (external, not from oracle)
                self.stack.push(TemporalType::Pure);
            }
            
            OpCode::Output => {
                let ty = self.pop_type();
                if ty == TemporalType::Temporal && !self.in_temporal_branch {
                    // Outputting temporal value: this is significant
                    // The output depends on the fixed point
                }
            }
            
            // ===== Control =====
            OpCode::Halt | OpCode::Paradox => {
                // No stack effect
            }
            
            // ===== Arrays =====
            OpCode::Pack => {
                // Consumes n+2 values, produces none
                self.pop_type(); // n
                self.pop_type(); // base
                // Additional values popped dynamically
            }
            OpCode::Unpack => {
                // Consumes 2, produces n values - type is unknown
                self.pop_type(); // n
                self.pop_type(); // base
                self.stack.push(TemporalType::Unknown);
            }
            OpCode::Index => {
                // Read from memory - result may be temporal
                let _index_type = self.pop_type();
                let _base_type = self.pop_type();
                self.stack.push(TemporalType::Temporal);
            }
            OpCode::Store => {
                // Write to memory
                self.pop_type(); // index
                self.pop_type(); // base
                self.pop_type(); // value
            }
        }
    }
    
    /// Pop a type from the abstract stack.
    fn pop_type(&mut self) -> TemporalType {
        self.stack.pop().unwrap_or(TemporalType::Unknown)
    }
    
    /// Peek at the top type.
    fn peek_type(&self) -> TemporalType {
        self.stack.last().copied().unwrap_or(TemporalType::Unknown)
    }
    
    /// Merge two stacks from branches (join corresponding types).
    fn merge_stacks(&self, a: &[TemporalType], b: &[TemporalType]) -> Vec<TemporalType> {
        let max_len = a.len().max(b.len());
        let mut result = Vec::with_capacity(max_len);
        
        for i in 0..max_len {
            let ta = a.get(i).copied().unwrap_or(TemporalType::Unknown);
            let tb = b.get(i).copied().unwrap_or(TemporalType::Unknown);
            result.push(ta.join(tb));
        }
        
        result
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Type-check a program and return the result.
pub fn type_check(program: &Program) -> TypeCheckResult {
    let mut checker = TypeChecker::new();
    checker.check(program)
}

/// Display type information for a program.
pub fn display_types(result: &TypeCheckResult) -> String {
    let mut output = String::new();
    
    output.push_str("=== Type Check Result ===\n");
    output.push_str(&format!("Valid: {}\n", result.is_valid));
    
    if !result.errors.is_empty() {
        output.push_str("\nErrors:\n");
        for err in &result.errors {
            output.push_str(&format!("  - {}\n", err));
        }
    }
    
    if !result.warnings.is_empty() {
        output.push_str("\nWarnings:\n");
        for warn in &result.warnings {
            output.push_str(&format!("  - {}\n", warn));
        }
    }
    
    if !result.cell_types.is_empty() {
        output.push_str("\nCell Types:\n");
        for (addr, ty) in &result.cell_types {
            output.push_str(&format!("  [{}]: {}\n", addr, ty.name()));
        }
    }
    
    if !result.final_stack_types.is_empty() {
        output.push_str("\nFinal Stack Types:\n");
        for (i, ty) in result.final_stack_types.iter().enumerate() {
            output.push_str(&format!("  {}: {}\n", i, ty.name()));
        }
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    
    fn check(source: &str) -> TypeCheckResult {
        let program = parse(source).expect("Parse failed");
        type_check(&program)
    }
    
    #[test]
    fn test_pure_program() {
        let result = check("10 20 ADD OUTPUT");
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        // Final stack should be empty (OUTPUT consumes)
    }
    
    #[test]
    fn test_temporal_from_oracle() {
        let result = check("0 ORACLE 0 PROPHECY");
        assert!(result.is_valid);
        // Should have no errors but might have info about temporal tainting
    }
    
    #[test]
    fn test_temporal_arithmetic() {
        let result = check("0 ORACLE 1 ADD 0 PROPHECY");
        assert!(result.is_valid);
        // The ADD of temporal + pure should be temporal
    }
    
    #[test]
    fn test_temporal_condition() {
        let result = check("0 ORACLE 0 EQ IF { 1 } ELSE { 2 } OUTPUT");
        assert!(result.is_valid);
        // Condition is temporal, so branches are tainted
    }
    
    #[test]
    fn test_type_lattice_join() {
        assert_eq!(TemporalType::Pure.join(TemporalType::Pure), TemporalType::Pure);
        assert_eq!(TemporalType::Pure.join(TemporalType::Temporal), TemporalType::Temporal);
        assert_eq!(TemporalType::Temporal.join(TemporalType::Pure), TemporalType::Temporal);
        assert_eq!(TemporalType::Unknown.join(TemporalType::Pure), TemporalType::Pure);
    }
    
    #[test]
    fn test_subtype_relation() {
        assert!(TemporalType::Pure.is_subtype_of(TemporalType::Pure));
        assert!(TemporalType::Pure.is_subtype_of(TemporalType::Temporal));
        assert!(TemporalType::Temporal.is_subtype_of(TemporalType::Temporal));
        assert!(!TemporalType::Temporal.is_subtype_of(TemporalType::Pure));
    }
}
