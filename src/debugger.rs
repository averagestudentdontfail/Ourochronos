//! Time-Travel Debugger for OUROCHRONOS.
//!
//! Provides epoch stepping, memory inspection, and temporal causality visualization.

use crate::ast::Program;
use crate::core_types::{Memory, Value, OutputItem};
use crate::vm::{Executor, ExecutorConfig, EpochStatus};

/// Debug event types.
#[derive(Debug, Clone)]
pub enum DebugEvent {
    /// Epoch started.
    EpochStart { epoch: usize },
    /// Epoch finished.
    EpochEnd { epoch: usize, status: EpochStatus },
    /// Memory read.
    MemoryRead { addr: u16, value: Value },
    /// Memory write.
    MemoryWrite { addr: u16, old_value: Value, new_value: Value },
    /// Stack operation.
    StackOp { op: String, stack_size: usize },
    /// Breakpoint hit.
    BreakpointHit { line: usize },
    /// Fixed point found.
    FixedPoint { epoch: usize },
    /// Paradox detected.
    Paradox { epoch: usize, reason: String },
}

/// Breakpoint.
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Unique ID.
    pub id: usize,
    /// Line number.
    pub line: usize,
    /// Condition (optional).
    pub condition: Option<String>,
    /// Enabled.
    pub enabled: bool,
}

/// Epoch snapshot for time-travel.
#[derive(Debug, Clone)]
pub struct EpochSnapshot {
    /// Epoch number.
    pub epoch: usize,
    /// Memory before epoch.
    pub anamnesis: Memory,
    /// Memory after epoch.
    pub present: Memory,
    /// Output produced.
    pub output: Vec<OutputItem>,
    /// Status.
    pub status: EpochStatus,
}

/// Time-travel debugger.
pub struct Debugger {
    /// Epoch history for time-travel.
    history: Vec<EpochSnapshot>,
    /// Current epoch index (for stepping through history).
    current_index: usize,
    /// Breakpoints.
    breakpoints: Vec<Breakpoint>,
    /// Event log.
    events: Vec<DebugEvent>,
    /// Maximum history size.
    _max_history: usize,
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}

impl Debugger {
    /// Create a new debugger.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            current_index: 0,
            breakpoints: Vec::new(),
            events: Vec::new(),
            _max_history: 1000,
        }
    }
    
    /// Run program with debugging.
    pub fn run(&mut self, program: &Program, anamnesis: Memory, max_epochs: usize) {
        self.history.clear();
        self.events.clear();
        self.current_index = 0;
        
        let mut executor = Executor::with_config(ExecutorConfig::default());
        let mut current_anamnesis = anamnesis;
        
        for epoch in 0..max_epochs {
            self.events.push(DebugEvent::EpochStart { epoch });
            
            let result = executor.run_epoch(program, &current_anamnesis);
            
            let snapshot = EpochSnapshot {
                epoch,
                anamnesis: current_anamnesis.clone(),
                present: result.present.clone(),
                output: result.output.clone(),
                status: result.status.clone(),
            };
            
            self.history.push(snapshot);
            self.current_index = self.history.len() - 1;
            
            self.events.push(DebugEvent::EpochEnd { 
                epoch, 
                status: result.status.clone() 
            });
            
            match result.status {
                EpochStatus::Finished => {
                    if result.present.values_equal(&current_anamnesis) {
                        self.events.push(DebugEvent::FixedPoint { epoch });
                        return;
                    }
                    current_anamnesis = result.present;
                }
                EpochStatus::Paradox => {
                    self.events.push(DebugEvent::Paradox { 
                        epoch, 
                        reason: "Explicit PARADOX".to_string() 
                    });
                    return;
                }
                EpochStatus::Error(_) | EpochStatus::Running => return,
            }
        }
    }
    
    /// Step back in time.
    pub fn step_back(&mut self) -> Option<&EpochSnapshot> {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
        self.history.get(self.current_index)
    }
    
    /// Step forward in time.
    pub fn step_forward(&mut self) -> Option<&EpochSnapshot> {
        if self.current_index + 1 < self.history.len() {
            self.current_index += 1;
        }
        self.history.get(self.current_index)
    }
    
    /// Jump to specific epoch.
    pub fn goto_epoch(&mut self, epoch: usize) -> Option<&EpochSnapshot> {
        if epoch < self.history.len() {
            self.current_index = epoch;
            self.history.get(self.current_index)
        } else {
            None
        }
    }
    
    /// Get current epoch snapshot.
    pub fn current(&self) -> Option<&EpochSnapshot> {
        self.history.get(self.current_index)
    }
    
    /// Get all snapshots.
    pub fn history(&self) -> &[EpochSnapshot] {
        &self.history
    }
    
    /// Add breakpoint.
    pub fn add_breakpoint(&mut self, line: usize) -> usize {
        let id = self.breakpoints.len();
        self.breakpoints.push(Breakpoint {
            id,
            line,
            condition: None,
            enabled: true,
        });
        id
    }
    
    /// Remove breakpoint.
    pub fn remove_breakpoint(&mut self, id: usize) {
        self.breakpoints.retain(|b| b.id != id);
    }
    
    /// Get breakpoints.
    pub fn breakpoints(&self) -> &[Breakpoint] {
        &self.breakpoints
    }
    
    /// Get events.
    pub fn events(&self) -> &[DebugEvent] {
        &self.events
    }
    
    /// Compare two epochs (show what changed).
    pub fn diff_epochs(&self, epoch1: usize, epoch2: usize) -> Vec<(u16, Value, Value)> {
        let mut changes = Vec::new();
        
        if let (Some(snap1), Some(snap2)) = (self.history.get(epoch1), self.history.get(epoch2)) {
            let mem1 = &snap1.present;
            let mem2 = &snap2.present;
            
            // Find all addresses that differ
            for addr in 0..256u16 {
                let v1 = mem1.read(addr);
                let v2 = mem2.read(addr);
                if v1.val != v2.val {
                    changes.push((addr, v1, v2));
                }
            }
        }
        
        changes
    }
    
    /// Get causality chain for a value.
    pub fn trace_causality(&self, addr: u16) -> Vec<(usize, Value)> {
        let mut chain = Vec::new();
        
        for (i, snap) in self.history.iter().enumerate() {
            let val = snap.present.read(addr);
            if chain.is_empty() || chain.last().map(|(_, v): &(usize, Value)| v.val != val.val).unwrap_or(false) {
                chain.push((i, val));
            }
        }
        
        chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    
    #[test]
    fn test_debugger_creation() {
        let debugger = Debugger::new();
        assert!(debugger.history.is_empty());
    }
    
    #[test]
    fn test_debugger_run() {
        let mut debugger = Debugger::new();
        let tokens = crate::parser::tokenize("1 2 ADD OUTPUT");
        let mut parser = crate::parser::Parser::new(&tokens);
        let program = parser.parse_program().unwrap();
        
        debugger.run(&program, Memory::new(), 10);
        
        assert!(!debugger.history.is_empty());
        assert!(!debugger.events.is_empty());
    }
    
    #[test]
    fn test_time_travel() {
        let mut debugger = Debugger::new();
        let tokens = crate::parser::tokenize("0 1 PROPHECY");
        let mut parser = crate::parser::Parser::new(&tokens);
        let program = parser.parse_program().unwrap();
        
        debugger.run(&program, Memory::new(), 5);
        
        // Should be able to step back
        let snap = debugger.step_back();
        assert!(snap.is_some() || debugger.history.len() <= 1);
    }
    
    #[test]
    fn test_breakpoints() {
        let mut debugger = Debugger::new();
        let id = debugger.add_breakpoint(5);
        assert_eq!(debugger.breakpoints().len(), 1);
        
        debugger.remove_breakpoint(id);
        assert!(debugger.breakpoints().is_empty());
    }
}
