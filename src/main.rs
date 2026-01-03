use ourochronos::{TimeLoop, ConvergenceStatus, Config, ExecutionMode, tokenize, Parser, ActionConfig, type_check, types};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: ourochronos <file.ouro> [options]");
        println!();
        println!("Options:");
        println!("  --diagnostic    Enable diagnostic mode (full trajectory recording)");
        println!("  --action        Enable action-guided mode (solves the Genie Effect)");
        println!("  --typecheck     Run static type analysis (temporal tainting)");
        println!("  --smt           Generate SMT-LIB2 output instead of running");
        println!("  --seed <n>      Set initial seed value");
        println!("  --seeds <n>     Number of seeds to try in action mode (default: 4)");
        return;
    }

    let filename = &args[1];
    let diagnostic = args.contains(&"--diagnostic".to_string());
    let action_mode = args.contains(&"--action".to_string());
    let smt = args.contains(&"--smt".to_string());
    let typecheck_mode = args.contains(&"--typecheck".to_string());
    
    // Parse seed: --seed <u64>
    let mut seed = 0;
    if let Some(idx) = args.iter().position(|a| a == "--seed") {
        if idx + 1 < args.len() {
             seed = args[idx+1].parse().unwrap_or(0);
        }
    }
    
    // Parse num_seeds: --seeds <usize>
    let mut num_seeds = 4;
    if let Some(idx) = args.iter().position(|a| a == "--seeds") {
        if idx + 1 < args.len() {
             num_seeds = args[idx+1].parse().unwrap_or(4);
        }
    }
    
    let source = fs::read_to_string(filename).expect("Failed to read file");
    
    let tokens = tokenize(&source);
    let mut parser = Parser::new(&tokens);
    parser.register_procedures(ourochronos::StdLib::procedures());
    
    match parser.parse_program() {
        Ok(parsed_program) => {
            // Inline all procedure calls
            let program = if !parsed_program.procedures.is_empty() {
                if diagnostic {
                    println!("Inlining {} procedure(s)...", parsed_program.procedures.len());
                }
                parsed_program.inline_procedures()
            } else {
                parsed_program
            };
            
            // Type check if requested
            if typecheck_mode {
                println!("=== Temporal Type Analysis ===");
                let result = type_check(&program);
                println!("{}", types::display_types(&result));
                if !result.is_valid {
                    eprintln!("Type errors found. Stopping.");
                    return;
                }
                println!(); // blank line before execution
            }
            
            if smt {
                let mut encoder = ourochronos::SmtEncoder::new();
                let smt_code = encoder.encode(&program);
                println!(";; Generated SMT-LIB2 for {}", filename);
                println!("{}", smt_code);
                return;
            }
        
            // Determine execution mode
            let mode = if action_mode {
                println!("Running in ACTION-GUIDED mode (exploring {} seeds).", num_seeds);
                ExecutionMode::ActionGuided {
                    config: ActionConfig::anti_trivial(),
                    num_seeds,
                }
            } else if diagnostic {
                println!("Running in DIAGNOSTIC mode.");
                ExecutionMode::Diagnostic
            } else {
                ExecutionMode::Standard
            };
            
            let config = Config {
                max_epochs: 1000,
                mode,
                seed,
                verbose: diagnostic || action_mode,
                frozen_inputs: Vec::new(),
            };
            
            let mut driver = TimeLoop::new(config.clone());
            match driver.run(&program) {
                ConvergenceStatus::Consistent { epochs, output, .. } => {
                    // Only print consistency status if verbose/diagnostic or NO output
                    if config.verbose || output.is_empty() {
                        println!("CONSISTENT after {} epochs.", epochs);
                    }
                    
                    if !output.is_empty() {
                         // Print raw output without label
                         for val in output {
                              if val.val >= 32 && val.val < 127 {
                                   print!("{}", val.val as u8 as char);
                              } else {
                                   print!("[{}]", val.val);
                              }
                         }
                         println!(); // Newline
                    }
                },
                ConvergenceStatus::Paradox { message, .. } => {
                    println!("PARADOX: {}", message);
                },
                ConvergenceStatus::Oscillation { period, oscillating_cells, diagnosis } => {
                    println!("OSCILLATION detected (period {})", period);
                    if diagnostic && !oscillating_cells.is_empty() {
                         println!("Oscillating Addresses: {:?}", oscillating_cells);
                    } else if diagnostic {
                        println!("No specific single-cell oscillations detected (global state cycle).");
                    }
                    
                    if diagnostic {
                        match diagnosis {
                            ourochronos::timeloop::ParadoxDiagnosis::NegativeLoop { explanation, .. } => {
                                println!("\nDIAGNOSIS (Grandfather Paradox):");
                                println!("{}", explanation);
                            },
                            ourochronos::timeloop::ParadoxDiagnosis::Oscillation { cycle } => {
                                println!("\nCycle states:");
                                for (i, state) in cycle.iter().enumerate() {
                                     // Only print non-empty states for brevity
                                     let non_zeros: Vec<_> = state.iter().filter(|(_,v)| *v != 0).collect();
                                     if !non_zeros.is_empty() {
                                         println!("  State {}: {:?}", i, non_zeros);
                                     }
                                }
                            },
                            ourochronos::timeloop::ParadoxDiagnosis::Unknown => {
                                println!("\nDIAGNOSIS: Unknown cause");
                            }
                        }
                    }
                },
                ConvergenceStatus::Timeout { max_epochs } => {
                    println!("TIMEOUT after {} epochs.", max_epochs);
                },
                ConvergenceStatus::Divergence { .. } => {
                     println!("DIVERGENCE detected.");
                },
                ConvergenceStatus::Error { message, .. } => {
                     println!("ERROR: {}", message);
                }
            }
        },
        Err(e) => {
            eprintln!("Parse Error: {}", e);
        }
    }
}
