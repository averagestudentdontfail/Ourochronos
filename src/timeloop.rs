use crate::core_types::{Memory};
use crate::ast::{Program};
use crate::vm::{Executor, EpochStatus};

#[derive(Debug, Clone)]
pub enum ConvergenceStatus {
    Consistent(Memory, usize), // Memory state, epochs count
    Paradox(String),
    Oscillation(usize, Memory, Vec<usize>), // Period, last state, oscillating addresses
    Timeout(usize),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Naive,       // Fast, stops at first oscillation or consistency
    Diagnostic,  // Records full trajectory
}

pub struct Config {
    pub max_epochs: usize,
    pub mode: ExecutionMode,
    pub seed: u64,
}

pub struct TimeLoop {
    pub config: Config,
    pub executor: Executor,
}

impl TimeLoop {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            executor: Executor::new(),
        }
    }

    pub fn run(&self, program: &Program) -> ConvergenceStatus {
        let mut anamnesis = Memory::new(); 
        
        // Seeded initialization (Addendum 2)
        if self.config.seed != 0 {
             // Simple fill for demo
             for i in 0..10 {
                 anamnesis.write(crate::Address(i as u16), crate::Value::new(self.config.seed.wrapping_add(i)));
             }
        }
        
        let mut history: Vec<Memory> = Vec::new();
        
        // In Diagnostic mode, we might want to pre-allocate
        if self.config.mode == ExecutionMode::Diagnostic {
            history.reserve(self.config.max_epochs);
        }

        for epoch in 0..self.config.max_epochs {
            // Check for cycles
            if let Some(pos) = history.iter().position(|m| m.is_equal_to(&anamnesis)) {
                 let period = history.len() - pos;
                 
                 // Diagnostic: Identify oscillating cells
                 let mut oscillating_addrs = Vec::new();
                 if self.config.mode == ExecutionMode::Diagnostic {
                     // Compare current `anamnesis` (which is equal to history[pos]) with elements in the loop
                     // Wait, the loop is history[pos]...history[end].
                     // Elements change within the loop.
                     // A cell oscillates if it takes >1 unique values in the loop range.
                     // But strictly, ANY cell that changes is oscillating if the state oscillates.
                     // Let's just report cells that differ between pos and pos+1 etc?
                     // Simpler: Compare min and max in the loop? Or just list all cells that are not constant.
                     
                     for addr in 0..crate::MEMORY_SIZE {
                         let base_val = history[pos].read(crate::Address(addr as u16)).val;
                         for step in pos+1..history.len() {
                             if history[step].read(crate::Address(addr as u16)).val != base_val {
                                 oscillating_addrs.push(addr);
                                 break;
                             }
                         }
                     }
                 }
                 
                 return ConvergenceStatus::Oscillation(period, anamnesis, oscillating_addrs);
            }
            
            // In Naive mode, we might optimize history storage (e.g. only hash), 
            // but for full correctness checking with limited epochs, full store is fine.
            history.push(anamnesis.clone());

            let (present, status) = self.executor.run_epoch(program, &anamnesis);
            
            match status {
                EpochStatus::Finished => {
                    // Check consistency: P == A?
                    if present.is_equal_to(&anamnesis) {
                        return ConvergenceStatus::Consistent(present, epoch + 1);
                    }
                    // Update: A_{n+1} = P_n
                    anamnesis = present;
                }
                EpochStatus::Paradox => {
                    return ConvergenceStatus::Paradox("Explicit PARADOX reached".to_string());
                }
                EpochStatus::Error(e) => {
                    return ConvergenceStatus::Paradox(format!("Runtime error: {}", e));
                }
                 _ => return ConvergenceStatus::Paradox("Unknown status".to_string()),
            }
        }
        
        ConvergenceStatus::Timeout(self.config.max_epochs)
    }
}
