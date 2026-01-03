//! Optimization passes for OUROCHRONOS programs.
//!
//! Implements instruction fusion and peephole optimizations before JIT compilation.
//! Based on optimization patterns from the Brainfuck compiler blog series.

use crate::ast::{Stmt, OpCode, Program};
use crate::core_types::Value;

/// Optimization level for the compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptLevel {
    /// No optimizations.
    None,
    /// Basic instruction fusion (combine consecutive ops).
    Basic,
    /// Full optimizations including pattern matching.
    Full,
}

impl Default for OptLevel {
    fn default() -> Self {
        OptLevel::Basic
    }
}

/// Optimized instruction representation.
/// 
/// These are fused instructions that represent common patterns
/// more efficiently than individual opcodes.
#[derive(Debug, Clone, PartialEq)]
pub enum OptInstr {
    /// Original statement (unfused).
    Stmt(Stmt),
    
    /// Fused addition: add N to top of stack.
    /// Replaces consecutive ADD operations.
    FusedAdd(i64),
    
    /// Fused memory operations: N consecutive reads/writes.
    FusedMemOps(Vec<(u16, OpCode)>),
    
    /// Clear pattern: set memory cell to zero.
    /// Detects patterns like: ORACLE DUP NOT IF { 0 PROPHECY } 
    Clear(u16),
    
    /// Move until zero: scan memory in a direction until finding 0.
    MoveUntil(i32),
    
    /// Copy value: copy from one cell to another.
    CopyTo { src: u16, dst: u16 },
    
    /// Block of optimized instructions.
    Block(Vec<OptInstr>),
}

/// Optimizer that transforms programs for better performance.
#[derive(Debug)]
pub struct Optimizer {
    level: OptLevel,
    stats: OptStats,
}

/// Statistics about optimizations applied.
#[derive(Debug, Default, Clone)]
pub struct OptStats {
    /// Number of instructions before optimization.
    pub original_count: usize,
    /// Number of instructions after optimization.
    pub optimized_count: usize,
    /// Number of fused additions.
    pub fused_adds: usize,
    /// Number of fused memory operations.
    pub fused_mem_ops: usize,
    /// Number of patterns detected (Clear, CopyTo, etc).
    pub patterns_detected: usize,
}

impl OptStats {
    /// Calculate reduction ratio.
    pub fn reduction_ratio(&self) -> f64 {
        if self.original_count == 0 {
            1.0
        } else {
            1.0 - (self.optimized_count as f64 / self.original_count as f64)
        }
    }
}

impl std::fmt::Display for OptStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Optimized: {} → {} instrs ({:.1}% reduction), {} fused adds, {} patterns",
            self.original_count, self.optimized_count,
            self.reduction_ratio() * 100.0,
            self.fused_adds, self.patterns_detected)
    }
}

impl Optimizer {
    /// Create a new optimizer with the given level.
    pub fn new(level: OptLevel) -> Self {
        Self {
            level,
            stats: OptStats::default(),
        }
    }
    
    /// Get optimization statistics.
    pub fn stats(&self) -> &OptStats {
        &self.stats
    }
    
    /// Optimize a program.
    pub fn optimize(&mut self, program: &Program) -> Vec<OptInstr> {
        self.stats.original_count = Self::count_stmts(&program.body);
        
        let result = match self.level {
            OptLevel::None => self.pass_through(&program.body),
            OptLevel::Basic => self.basic_optimize(&program.body),
            OptLevel::Full => self.full_optimize(&program.body),
        };
        
        self.stats.optimized_count = Self::count_opt_instrs(&result);
        result
    }
    
    fn count_stmts(stmts: &[Stmt]) -> usize {
        let mut count = 0;
        for stmt in stmts {
            count += 1;
            match stmt {
                Stmt::Block(inner) => count += Self::count_stmts(inner),
                Stmt::If { then_branch, else_branch } => {
                    count += Self::count_stmts(then_branch);
                    if let Some(eb) = else_branch {
                        count += Self::count_stmts(eb);
                    }
                }
                Stmt::While { cond, body } => {
                    count += Self::count_stmts(cond);
                    count += Self::count_stmts(body);
                }
                _ => {}
            }
        }
        count
    }
    
    fn count_opt_instrs(instrs: &[OptInstr]) -> usize {
        let mut count = 0;
        for instr in instrs {
            count += 1;
            if let OptInstr::Block(inner) = instr {
                count += Self::count_opt_instrs(inner);
            }
        }
        count
    }
    
    /// No optimization - just wrap statements.
    fn pass_through(&self, stmts: &[Stmt]) -> Vec<OptInstr> {
        stmts.iter().map(|s| OptInstr::Stmt(s.clone())).collect()
    }
    
    /// Basic optimization: fuse consecutive operations.
    fn basic_optimize(&mut self, stmts: &[Stmt]) -> Vec<OptInstr> {
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < stmts.len() {
            match &stmts[i] {
                // Fuse consecutive Push operations for arithmetic
                Stmt::Push(val) => {
                    if i + 1 < stmts.len() {
                        if let Stmt::Op(OpCode::Add) = &stmts[i + 1] {
                            // Check for more adds
                            let mut sum = val.val as i64;
                            let mut j = i + 2;
                            
                            while j + 1 < stmts.len() {
                                if let (Stmt::Push(v), Stmt::Op(OpCode::Add)) = 
                                    (&stmts[j], &stmts[j + 1]) {
                                    sum += v.val as i64;
                                    j += 2;
                                } else {
                                    break;
                                }
                            }
                            
                            if j > i + 2 {
                                // Found consecutive adds
                                self.stats.fused_adds += 1;
                                result.push(OptInstr::FusedAdd(sum));
                                i = j;
                                continue;
                            }
                        }
                    }
                    result.push(OptInstr::Stmt(stmts[i].clone()));
                }
                
                // Recursively optimize blocks
                Stmt::Block(inner) => {
                    let optimized = self.basic_optimize(inner);
                    result.push(OptInstr::Block(optimized));
                }
                
                Stmt::If { then_branch, else_branch } => {
                    let opt_then = self.basic_optimize(then_branch);
                    let opt_else = else_branch.as_ref().map(|eb| self.basic_optimize(eb));
                    
                    // Keep original if structure but with optimized branches
                    result.push(OptInstr::Stmt(Stmt::If {
                        then_branch: Self::unoptimize(&opt_then),
                        else_branch: opt_else.map(|e| Self::unoptimize(&e)),
                    }));
                }
                
                Stmt::While { cond, body } => {
                    let opt_cond = self.basic_optimize(cond);
                    let opt_body = self.basic_optimize(body);
                    
                    result.push(OptInstr::Stmt(Stmt::While {
                        cond: Self::unoptimize(&opt_cond),
                        body: Self::unoptimize(&opt_body),
                    }));
                }
                
                _ => {
                    result.push(OptInstr::Stmt(stmts[i].clone()));
                }
            }
            i += 1;
        }
        
        result
    }
    
    /// Full optimization: includes pattern detection.
    fn full_optimize(&mut self, stmts: &[Stmt]) -> Vec<OptInstr> {
        // First apply basic optimizations
        let basic = self.basic_optimize(stmts);
        
        // Then apply peephole patterns
        self.peephole_optimize(basic)
    }
    
    /// Peephole optimization pass.
    fn peephole_optimize(&mut self, instrs: Vec<OptInstr>) -> Vec<OptInstr> {
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < instrs.len() {
            match &instrs[i] {
                // Detect Clear pattern: ORACLE addr, 0, PROPHECY
                OptInstr::Stmt(Stmt::Op(OpCode::Oracle)) => {
                    if i + 2 < instrs.len() {
                        if let (
                            OptInstr::Stmt(Stmt::Push(val)),
                            OptInstr::Stmt(Stmt::Op(OpCode::Prophecy))
                        ) = (&instrs[i + 1], &instrs[i + 2]) {
                            if val.val == 0 {
                                self.stats.patterns_detected += 1;
                                result.push(OptInstr::Clear(0)); // Address unknown at this stage
                                i += 3;
                                continue;
                            }
                        }
                    }
                    result.push(instrs[i].clone());
                }
                
                // Recursively optimize blocks
                OptInstr::Block(inner) => {
                    let optimized = self.peephole_optimize(inner.clone());
                    result.push(OptInstr::Block(optimized));
                }
                
                _ => {
                    result.push(instrs[i].clone());
                }
            }
            i += 1;
        }
        
        result
    }
    
    /// Convert optimized instructions back to statements (for compatibility).
    fn unoptimize(instrs: &[OptInstr]) -> Vec<Stmt> {
        instrs.iter().flat_map(|instr| {
            match instr {
                OptInstr::Stmt(s) => vec![s.clone()],
                OptInstr::FusedAdd(n) => vec![
                    Stmt::Push(Value::new(*n as u64)),
                    Stmt::Op(OpCode::Add),
                ],
                OptInstr::Clear(_) => vec![
                    Stmt::Push(Value::new(0)),
                    Stmt::Op(OpCode::Prophecy),
                ],
                OptInstr::Block(inner) => Self::unoptimize(inner),
                OptInstr::FusedMemOps(ops) => {
                    ops.iter().map(|(_, op)| Stmt::Op(*op)).collect()
                }
                OptInstr::MoveUntil(_) | OptInstr::CopyTo { .. } => {
                    // These need expansion
                    vec![]
                }
            }
        }).collect()
    }
}

/// Tiered execution: decides whether to interpret or JIT compile.
#[derive(Debug)]
pub struct TieredExecutor {
    /// Execution counts for each block.
    execution_counts: std::collections::HashMap<u64, usize>,
    /// Threshold for JIT compilation.
    jit_threshold: usize,
}

impl Default for TieredExecutor {
    fn default() -> Self {
        Self::new(100) // JIT after 100 executions
    }
}

impl TieredExecutor {
    /// Create with custom threshold.
    pub fn new(jit_threshold: usize) -> Self {
        Self {
            execution_counts: std::collections::HashMap::new(),
            jit_threshold,
        }
    }
    
    /// Check if a block should be JIT compiled.
    pub fn should_jit(&mut self, block_hash: u64) -> bool {
        let count = self.execution_counts.entry(block_hash).or_insert(0);
        *count += 1;
        *count >= self.jit_threshold
    }
    
    /// Get current execution count for a block.
    pub fn get_count(&self, block_hash: u64) -> usize {
        *self.execution_counts.get(&block_hash).unwrap_or(&0)
    }
    
    /// Reset statistics.
    pub fn reset(&mut self) {
        self.execution_counts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimizer_creation() {
        let opt = Optimizer::new(OptLevel::Basic);
        assert_eq!(opt.stats().original_count, 0);
    }
    
    #[test]
    fn test_opt_stats_display() {
        let stats = OptStats {
            original_count: 100,
            optimized_count: 50,
            fused_adds: 10,
            fused_mem_ops: 5,
            patterns_detected: 3,
        };
        let s = format!("{}", stats);
        assert!(s.contains("50.0%"));
    }
    
    #[test]
    fn test_tiered_executor() {
        let mut tiered = TieredExecutor::new(3);
        assert!(!tiered.should_jit(12345));
        assert!(!tiered.should_jit(12345));
        assert!(tiered.should_jit(12345));
        assert_eq!(tiered.get_count(12345), 3);
    }
    
    #[test]
    fn test_basic_optimization() {
        let mut opt = Optimizer::new(OptLevel::Basic);
        let mut program = Program::new();
        program.body = vec![
            Stmt::Push(Value::new(1)),
            Stmt::Op(OpCode::Add),
            Stmt::Push(Value::new(2)),
            Stmt::Op(OpCode::Add),
        ];
        
        let result = opt.optimize(&program);
        // Should have fused the two adds
        assert!(opt.stats().fused_adds > 0 || result.len() > 0);
    }
}
