//! Fixed-point computation and temporal loop execution.
//!
//! This module implements the core OUROCHRONOS execution model:
//! repeatedly run epochs until Present = Anamnesis (fixed point achieved).
//!
//! The module also provides paradox diagnosis when no fixed point exists.

use crate::core_types::{Memory, Address, Value};
use crate::ast::Program;
use crate::vm::{Executor, ExecutorConfig, EpochStatus};
use std::collections::HashMap;

/// Result of fixed-point search.
#[derive(Debug)]
pub enum ConvergenceStatus {
    /// Fixed point found: P = A.
    Consistent {
        /// The consistent memory state.
        memory: Memory,
        /// Output produced.
        output: Vec<Value>,
        /// Number of epochs to converge.
        epochs: usize,
    },
    
    /// Explicit PARADOX instruction reached.
    Paradox {
        /// Diagnostic message.
        message: String,
        /// Epoch where paradox occurred.
        epoch: usize,
    },
    
    /// Oscillation detected (cycle of length > 1).
    Oscillation {
        /// Cycle period.
        period: usize,
        /// Addresses that oscillate.
        oscillating_cells: Vec<Address>,
        /// Diagnosis of the paradox.
        diagnosis: ParadoxDiagnosis,
    },
    
    /// Divergence detected (monotonic unbounded growth).
    Divergence {
        /// Cells that diverge.
        diverging_cells: Vec<Address>,
        /// Direction of divergence.
        direction: Direction,
    },
    
    /// Epoch limit reached without convergence.
    Timeout {
        /// Maximum epochs attempted.
        max_epochs: usize,
    },
    
    /// Runtime error during execution.
    Error {
        message: String,
        epoch: usize,
    },
}

/// Direction of divergence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Increasing,
    Decreasing,
}

/// Diagnosis of why a paradox occurred.
#[derive(Debug, Clone)]
pub enum ParadoxDiagnosis {
    /// Negative causal loop (grandfather paradox).
    NegativeLoop {
        /// Cells involved in the loop.
        cells: Vec<Address>,
        /// Human-readable explanation.
        explanation: String,
    },
    
    /// General oscillation.
    Oscillation {
        /// Cycle of memory states.
        cycle: Vec<Vec<(Address, u64)>>,
    },
    
    /// Unknown cause.
    Unknown,
}

/// Execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Basic iteration with cycle detection.
    Standard,
    /// Full trajectory recording and analysis.
    Diagnostic,
    /// Unbounded iteration (may not terminate).
    Pure,
}

/// Configuration for the time loop.
#[derive(Debug, Clone)]
pub struct TimeLoopConfig {
    /// Maximum epochs before timeout.
    pub max_epochs: usize,
    /// Execution mode.
    pub mode: ExecutionMode,
    /// Initial seed for anamnesis (0 = all zeros).
    pub seed: u64,
    /// Whether to print progress.
    pub verbose: bool,
}

impl Default for TimeLoopConfig {
    fn default() -> Self {
        Self {
            max_epochs: 10_000,
            mode: ExecutionMode::Standard,
            seed: 0,
            verbose: false,
        }
    }
}

/// The temporal loop driver.
pub struct TimeLoop {
    config: TimeLoopConfig,
    executor: Executor,
}

impl TimeLoop {
    /// Create a new time loop with given configuration.
    pub fn new(config: TimeLoopConfig) -> Self {
        let mut exec_config = ExecutorConfig::default();
        exec_config.immediate_output = config.verbose;
        
        Self {
            config,
            executor: Executor::with_config(exec_config),
        }
    }
    
    /// Run the fixed-point search.
    pub fn run(&mut self, program: &Program) -> ConvergenceStatus {
        // Check for trivial consistency
        if program.is_trivially_consistent() {
            return self.run_trivial(program);
        }
        
        match self.config.mode {
            ExecutionMode::Standard => self.run_standard(program),
            ExecutionMode::Diagnostic => self.run_diagnostic(program),
            ExecutionMode::Pure => self.run_pure(program),
        }
    }
    
    /// Run a trivially consistent program (no oracle operations).
    fn run_trivial(&mut self, program: &Program) -> ConvergenceStatus {
        let result = self.executor.run_epoch(program, &Memory::new());
        
        match result.status {
            EpochStatus::Finished => ConvergenceStatus::Consistent {
                memory: result.present,
                output: result.output,
                epochs: 1,
            },
            EpochStatus::Paradox => ConvergenceStatus::Paradox {
                message: "Explicit PARADOX in trivial program".to_string(),
                epoch: 1,
            },
            EpochStatus::Error(e) => ConvergenceStatus::Error {
                message: e,
                epoch: 1,
            },
            _ => unreachable!(),
        }
    }
    
    /// Standard execution with cycle detection.
    fn run_standard(&mut self, program: &Program) -> ConvergenceStatus {
        let mut anamnesis = self.create_initial_anamnesis();
        let mut seen_states: HashMap<u64, usize> = HashMap::new();
        
        for epoch in 0..self.config.max_epochs {
            let state_hash = anamnesis.state_hash();
            
            // Check for cycle
            if let Some(&previous_epoch) = seen_states.get(&state_hash) {
                let period = epoch - previous_epoch;
                return self.diagnose_oscillation(program, &anamnesis, period);
            }
            
            seen_states.insert(state_hash, epoch);
            
            // Run epoch
            let result = self.executor.run_epoch(program, &anamnesis);
            
            match result.status {
                EpochStatus::Finished => {
                    // Check for fixed point
                    if result.present.values_equal(&anamnesis) {
                        return ConvergenceStatus::Consistent {
                            memory: result.present,
                            output: result.output,
                            epochs: epoch + 1,
                        };
                    }
                    
                    // Continue iteration
                    anamnesis = result.present;
                }
                
                EpochStatus::Paradox => {
                    return ConvergenceStatus::Paradox {
                        message: "Explicit PARADOX instruction".to_string(),
                        epoch: epoch + 1,
                    };
                }
                
                EpochStatus::Error(e) => {
                    return ConvergenceStatus::Error {
                        message: e,
                        epoch: epoch + 1,
                    };
                }
                
                _ => {}
            }
        }
        
        ConvergenceStatus::Timeout {
            max_epochs: self.config.max_epochs,
        }
    }
    
    /// Diagnostic execution with full trajectory recording.
    fn run_diagnostic(&mut self, program: &Program) -> ConvergenceStatus {
        let mut anamnesis = self.create_initial_anamnesis();
        let mut trajectory: Vec<Memory> = Vec::new();
        let mut seen_states: HashMap<u64, usize> = HashMap::new();
        
        for epoch in 0..self.config.max_epochs {
            let state_hash = anamnesis.state_hash();
            
            // Check for cycle
            if let Some(&previous_epoch) = seen_states.get(&state_hash) {
                let period = epoch - previous_epoch;
                let oscillating = self.find_oscillating_cells(&trajectory[previous_epoch..]);
                let diagnosis = self.create_oscillation_diagnosis(&trajectory[previous_epoch..]);
                
                return ConvergenceStatus::Oscillation {
                    period,
                    oscillating_cells: oscillating,
                    diagnosis,
                };
            }
            
            seen_states.insert(state_hash, epoch);
            trajectory.push(anamnesis.clone());
            
            // Check for divergence (monotonic growth)
            if epoch > 10 {
                if let Some((cells, direction)) = self.detect_divergence(&trajectory) {
                    return ConvergenceStatus::Divergence {
                        diverging_cells: cells,
                        direction,
                    };
                }
            }
            
            // Run epoch
            let result = self.executor.run_epoch(program, &anamnesis);
            
            match result.status {
                EpochStatus::Finished => {
                    if result.present.values_equal(&anamnesis) {
                        return ConvergenceStatus::Consistent {
                            memory: result.present,
                            output: result.output,
                            epochs: epoch + 1,
                        };
                    }
                    anamnesis = result.present;
                }
                
                EpochStatus::Paradox => {
                    return ConvergenceStatus::Paradox {
                        message: "Explicit PARADOX instruction".to_string(),
                        epoch: epoch + 1,
                    };
                }
                
                EpochStatus::Error(e) => {
                    return ConvergenceStatus::Error {
                        message: e,
                        epoch: epoch + 1,
                    };
                }
                
                _ => {}
            }
        }
        
        ConvergenceStatus::Timeout {
            max_epochs: self.config.max_epochs,
        }
    }
    
    /// Pure execution (unbounded, for theoretical exploration).
    fn run_pure(&mut self, program: &Program) -> ConvergenceStatus {
        let mut anamnesis = self.create_initial_anamnesis();
        let mut epoch = 0;
        
        loop {
            let result = self.executor.run_epoch(program, &anamnesis);
            epoch += 1;
            
            match result.status {
                EpochStatus::Finished => {
                    if result.present.values_equal(&anamnesis) {
                        return ConvergenceStatus::Consistent {
                            memory: result.present,
                            output: result.output,
                            epochs: epoch,
                        };
                    }
                    anamnesis = result.present;
                }
                
                EpochStatus::Paradox => {
                    return ConvergenceStatus::Paradox {
                        message: "Explicit PARADOX instruction".to_string(),
                        epoch,
                    };
                }
                
                EpochStatus::Error(e) => {
                    return ConvergenceStatus::Error {
                        message: e,
                        epoch,
                    };
                }
                
                _ => {}
            }
            
            // Safety check for interactive use
            if epoch > 1_000_000 {
                return ConvergenceStatus::Timeout { max_epochs: epoch };
            }
        }
    }
    
    /// Create initial anamnesis based on seed.
    fn create_initial_anamnesis(&self) -> Memory {
        let mut mem = Memory::new();
        
        if self.config.seed != 0 {
            // Seed first few cells for variety
            for i in 0..16 {
                let val = self.config.seed.wrapping_mul(i as u64 + 1);
                mem.write(i as Address, Value::new(val));
            }
        }
        
        mem
    }
    
    /// Diagnose oscillation and identify oscillating cells.
    fn diagnose_oscillation(&mut self, program: &Program, 
                           anamnesis: &Memory, period: usize) -> ConvergenceStatus {
        // Re-run to capture the cycle
        let mut states = Vec::new();
        let mut current = anamnesis.clone();
        
        for _ in 0..period + 1 {
            states.push(current.clone());
            let result = self.executor.run_epoch(program, &current);
            current = result.present;
        }
        
        let oscillating = self.find_oscillating_cells(&states);
        let diagnosis = self.create_oscillation_diagnosis(&states);
        
        ConvergenceStatus::Oscillation {
            period,
            oscillating_cells: oscillating,
            diagnosis,
        }
    }
    
    /// Find cells that change within a cycle.
    fn find_oscillating_cells(&self, states: &[Memory]) -> Vec<Address> {
        if states.is_empty() {
            return Vec::new();
        }
        
        let mut oscillating = Vec::new();
        let first = &states[0];
        
        for addr in 0..256u16 { // Check first 256 cells for efficiency
            let base_val = first.read(addr).val;
            for state in states.iter().skip(1) {
                if state.read(addr).val != base_val {
                    oscillating.push(addr);
                    break;
                }
            }
        }
        
        oscillating
    }
    
    /// Create a diagnosis of the oscillation.
    fn create_oscillation_diagnosis(&self, states: &[Memory]) -> ParadoxDiagnosis {
        if states.len() == 2 {
            // Period-2 oscillation: likely grandfather paradox
            let diffs = states[0].diff(&states[1]);
            
            if diffs.len() == 1 {
                let addr = diffs[0];
                let v1 = states[0].read(addr).val;
                let v2 = states[1].read(addr).val;
                
                // Check for negation pattern
                if v1 == !v2 || (v1 == 0 && v2 != 0) || (v1 != 0 && v2 == 0) {
                    return ParadoxDiagnosis::NegativeLoop {
                        cells: vec![addr],
                        explanation: format!(
                            "Cell {} oscillates between {} and {}. \
                             This is a grandfather paradox: the cell's value \
                             determines its own opposite.",
                            addr, v1, v2
                        ),
                    };
                }
            }
        }
        
        // General oscillation
        let cycle: Vec<Vec<(Address, u64)>> = states.iter()
            .map(|s| s.non_zero_cells().iter()
                .map(|(a, v)| (*a, v.val))
                .collect())
            .collect();
        
        ParadoxDiagnosis::Oscillation { cycle }
    }
    
    /// Detect divergence (monotonic unbounded growth).
    fn detect_divergence(&self, trajectory: &[Memory]) -> Option<(Vec<Address>, Direction)> {
        if trajectory.len() < 5 {
            return None;
        }
        
        let mut diverging = Vec::new();
        let mut direction = Direction::Increasing;
        
        // Check each cell for monotonic growth
        for addr in 0..256u16 {
            let values: Vec<u64> = trajectory.iter()
                .map(|m| m.read(addr).val)
                .collect();
            
            // Check if strictly increasing
            let increasing = values.windows(2).all(|w| w[1] > w[0]);
            let decreasing = values.windows(2).all(|w| w[1] < w[0]);
            
            if increasing {
                diverging.push(addr);
                direction = Direction::Increasing;
            } else if decreasing {
                diverging.push(addr);
                direction = Direction::Decreasing;
            }
        }
        
        if diverging.is_empty() {
            None
        } else {
            Some((diverging, direction))
        }
    }
}

/// Format convergence status for display.
pub fn format_status(status: &ConvergenceStatus) -> String {
    match status {
        ConvergenceStatus::Consistent { epochs, output, .. } => {
            let mut s = format!("CONSISTENT after {} epoch(s)\n", epochs);
            if !output.is_empty() {
                s.push_str(&format!("Output: {:?}\n", 
                    output.iter().map(|v| v.val).collect::<Vec<_>>()));
            }
            s
        }
        
        ConvergenceStatus::Paradox { message, epoch } => {
            format!("PARADOX at epoch {}: {}\n", epoch, message)
        }
        
        ConvergenceStatus::Oscillation { period, oscillating_cells, diagnosis } => {
            let mut s = format!("OSCILLATION detected (period {})\n", period);
            s.push_str(&format!("Oscillating cells: {:?}\n", oscillating_cells));
            
            match diagnosis {
                ParadoxDiagnosis::NegativeLoop { explanation, .. } => {
                    s.push_str(&format!("\nDIAGNOSIS (Grandfather Paradox):\n{}\n", explanation));
                }
                ParadoxDiagnosis::Oscillation { cycle } => {
                    s.push_str("\nCycle states:\n");
                    for (i, state) in cycle.iter().enumerate() {
                        if !state.is_empty() {
                            s.push_str(&format!("  State {}: {:?}\n", i, state));
                        }
                    }
                }
                ParadoxDiagnosis::Unknown => {
                    s.push_str("\nDIAGNOSIS: Unknown cause\n");
                }
            }
            s
        }
        
        ConvergenceStatus::Divergence { diverging_cells, direction } => {
            format!("DIVERGENCE detected\n\
                    Diverging cells: {:?}\n\
                    Direction: {:?}\n\
                    \n\
                    DIAGNOSIS: Cell value(s) grow without bound.\n\
                    No fixed point is reachable.\n",
                    diverging_cells, direction)
        }
        
        ConvergenceStatus::Timeout { max_epochs } => {
            format!("TIMEOUT after {} epochs\n", max_epochs)
        }
        
        ConvergenceStatus::Error { message, epoch } => {
            format!("ERROR at epoch {}: {}\n", epoch, message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    
    fn run_program(source: &str) -> ConvergenceStatus {
        let program = parse(source).expect("Parse failed");
        let config = TimeLoopConfig::default();
        let mut driver = TimeLoop::new(config);
        driver.run(&program)
    }
    
    #[test]
    fn test_trivial_consistency() {
        let status = run_program("10 20 ADD OUTPUT");
        assert!(matches!(status, ConvergenceStatus::Consistent { epochs: 1, .. }));
    }
    
    #[test]
    fn test_self_fulfilling() {
        // Oracle reads 0, writes 0 → consistent immediately
        let status = run_program("0 ORACLE 0 PROPHECY");
        assert!(matches!(status, ConvergenceStatus::Consistent { .. }));
    }
    
    #[test]
    fn test_grandfather_paradox() {
        // Reads A[0], writes NOT(A[0]) → oscillates
        let status = run_program("0 ORACLE NOT 0 PROPHECY");
        assert!(matches!(status, ConvergenceStatus::Oscillation { period: 2, .. }));
    }
    
    #[test]
    fn test_divergence() {
        // Reads A[0], writes A[0]+1 → diverges
        let status = run_program("0 ORACLE 1 ADD 0 PROPHECY");
        
        // Either detected as divergence or timeout
        assert!(matches!(status, 
            ConvergenceStatus::Divergence { .. } | 
            ConvergenceStatus::Timeout { .. }
        ));
    }
}
