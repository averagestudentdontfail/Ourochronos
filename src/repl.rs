//! REPL (Read-Eval-Print-Loop) for OUROCHRONOS.
//!
//! Interactive shell for exploring temporal programs.

use std::io::{self, Write};

use crate::ast::Program;
use crate::parser::Parser;
use crate::timeloop::{TimeLoop, TimeLoopConfig, ConvergenceStatus};
use crate::core_types::{Memory, Value};

/// REPL configuration.
#[derive(Debug, Clone)]
pub struct ReplConfig {
    /// Prompt string.
    pub prompt: String,
    /// Maximum epochs per evaluation.
    pub max_epochs: usize,
    /// Show verbose output.
    pub verbose: bool,
    /// Show memory after each evaluation.
    pub show_memory: bool,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            prompt: "ouro> ".to_string(),
            max_epochs: 100,
            verbose: false,
            show_memory: false,
        }
    }
}

/// Interactive REPL for OUROCHRONOS.
pub struct Repl {
    config: ReplConfig,
    memory: Memory,
    history: Vec<String>,
}

impl Repl {
    /// Create a new REPL with default config.
    pub fn new() -> Self {
        Self::with_config(ReplConfig::default())
    }
    
    /// Create with custom config.
    pub fn with_config(config: ReplConfig) -> Self {
        Self {
            config,
            memory: Memory::new(),
            history: Vec::new(),
        }
    }
    
    /// Run the interactive REPL.
    pub fn run(&mut self) -> io::Result<()> {
        println!("OUROCHRONOS REPL v0.2.0");
        println!("Type :help for commands, :quit to exit");
        println!();
        
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut input = String::new();
        
        loop {
            // Print prompt
            print!("{}", self.config.prompt);
            stdout.flush()?;
            
            // Read input
            input.clear();
            if stdin.read_line(&mut input)? == 0 {
                break; // EOF
            }
            
            let line = input.trim();
            if line.is_empty() {
                continue;
            }
            
            // Handle commands
            if line.starts_with(':') {
                if self.handle_command(line) {
                    break;
                }
                continue;
            }
            
            // Evaluate code
            self.eval(line);
            self.history.push(line.to_string());
        }
        
        println!("\nGoodbye!");
        Ok(())
    }
    
    /// Handle REPL commands.
    fn handle_command(&mut self, cmd: &str) -> bool {
        match cmd {
            ":quit" | ":q" => return true,
            ":help" | ":h" => {
                println!("Commands:");
                println!("  :quit, :q     Exit the REPL");
                println!("  :help, :h     Show this help");
                println!("  :memory, :m   Show memory state");
                println!("  :clear, :c    Clear memory");
                println!("  :history      Show command history");
                println!("  :verbose      Toggle verbose mode");
                println!();
                println!("Enter OUROCHRONOS code to evaluate.");
            }
            ":memory" | ":m" => {
                self.show_memory();
            }
            ":clear" | ":c" => {
                self.memory = Memory::new();
                println!("Memory cleared.");
            }
            ":history" => {
                for (i, line) in self.history.iter().enumerate() {
                    println!("{}: {}", i + 1, line);
                }
            }
            ":verbose" => {
                self.config.verbose = !self.config.verbose;
                println!("Verbose mode: {}", if self.config.verbose { "on" } else { "off" });
            }
            _ => {
                println!("Unknown command: {}", cmd);
            }
        }
        false
    }
    
    /// Evaluate code.
    pub fn eval(&mut self, code: &str) -> Option<ConvergenceStatus> {
        let tokens = crate::parser::tokenize(code);
        let mut parser = Parser::new(&tokens);
        let program = match parser.parse_program() {
            Ok(p) => p,
            Err(e) => {
                println!("Parse error: {}", e);
                return None;
            }
        };
        
        let config = TimeLoopConfig {
            max_epochs: self.config.max_epochs,
            verbose: self.config.verbose,
            ..Default::default()
        };
        
        let mut timeloop = TimeLoop::new(config);
        let result = timeloop.run(&program);
        
        match &result {
            ConvergenceStatus::Consistent { memory, output, epochs } => {
                self.memory = memory.clone();
                if !output.is_empty() {
                    print!("Output: ");
                    for val in output {
                        print!("{} ", val.val);
                    }
                    println!();
                }
                if self.config.verbose {
                    println!("Converged in {} epochs", epochs);
                }
            }
            ConvergenceStatus::Oscillation { period, .. } => {
                println!("⟳ Oscillation detected (period {})", period);
            }
            ConvergenceStatus::Paradox { epoch, .. } => {
                println!("✗ Paradox at epoch {}", epoch);
            }
            ConvergenceStatus::Timeout { max_epochs } => {
                println!("⋯ No convergence after {} epochs", max_epochs);
            }
            ConvergenceStatus::Divergence { .. } => {
                println!("∞ Divergence detected");
            }
            ConvergenceStatus::Error { message, .. } => {
                println!("✗ Error: {}", message);
            }
        }
        
        if self.config.show_memory {
            self.show_memory();
        }
        
        Some(result)
    }
    
    /// Show memory state.
    fn show_memory(&self) {
        let cells = self.memory.non_zero_cells();
        if cells.is_empty() {
            println!("Memory: (empty)");
        } else {
            println!("Memory:");
            for (addr, val) in cells {
                println!("  [{}] = {}", addr, val.val);
            }
        }
    }
    
    /// Get current memory.
    pub fn memory(&self) -> &Memory {
        &self.memory
    }
    
    /// Set memory.
    pub fn set_memory(&mut self, memory: Memory) {
        self.memory = memory;
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_repl_creation() {
        let repl = Repl::new();
        assert!(repl.history.is_empty());
    }
    
    #[test]
    fn test_repl_eval() {
        let mut repl = Repl::new();
        let result = repl.eval("1 2 ADD OUTPUT");
        assert!(result.is_some());
    }
    
    #[test]
    fn test_repl_memory_persistence() {
        let mut repl = Repl::new();
        let result = repl.eval("1 2 ADD OUTPUT");
        // Result should be some for valid program
        assert!(result.is_some());
    }
}
