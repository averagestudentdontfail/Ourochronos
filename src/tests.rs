#[cfg(test)]
mod tests {
    use crate::*;

    fn parse(code: &str) -> Program {
        let tokens = tokenize(code);
        let mut parser = Parser::new(&tokens);
        parser.parse_program().expect("Failed to parse program")
    }

    fn default_config() -> Config {
        Config {
            max_epochs: 100,
            mode: ExecutionMode::Standard,
            seed: 0,
            verbose: false,
        }
    }

    #[test]
    fn test_trivial_consistency() {
        let program = parse("10 20 ADD OUTPUT");
        let result = TimeLoop::new(default_config()).run(&program);
        assert!(matches!(result, ConvergenceStatus::Consistent { epochs: 1, .. }));
    }
    
    #[test]
    fn test_self_fulfilling_prophecy() {
        let program = parse("0 ORACLE 0 PROPHECY");
        let result = TimeLoop::new(default_config()).run(&program);
        assert!(matches!(result, ConvergenceStatus::Consistent { .. }));
    }
    
    #[test]
    fn test_grandfather_paradox() {
        let program = parse("0 ORACLE NOT 0 PROPHECY");
        let result = TimeLoop::new(default_config()).run(&program);
        if let ConvergenceStatus::Oscillation { period, .. } = result {
            assert_eq!(period, 2);
        } else {
            panic!("Expected Oscillation, got {:?}", result);
        }
    }
    
    #[test]
    fn test_divergence() {
        let program = parse("0 ORACLE 1 ADD 0 PROPHECY");
        let result = TimeLoop::new(Config { max_epochs: 50, ..default_config() }).run(&program);
        assert!(matches!(result, ConvergenceStatus::Timeout { .. } | ConvergenceStatus::Divergence { .. }));
    }
    
    #[test]
    fn test_witness_pattern_primality() {
        let program = parse("0 ORACLE DUP 3 EQ IF { DUP 0 PROPHECY } ELSE { POP 3 0 PROPHECY }");
        let result = TimeLoop::new(default_config()).run(&program);
        
        match result {
            ConvergenceStatus::Consistent { memory, epochs, .. } => {
                assert_eq!(memory.read(0).val, 3);
                assert!(epochs <= 2);
            }
            _ => panic!("Expected consistent execution, got {:?}", result),
        }
    }

    #[test]
    fn test_smt_generation_smoke() {
        let program = parse("0 ORACLE DUP 1 ADD 0 PROPHECY");
        let mut encoder = SmtEncoder::new();
        let smt = encoder.encode(&program);
        assert!(smt.contains("(declare-const anamnesis"));
        assert!(smt.contains("(check-sat)"));
    }
    
    #[test]
    fn test_smt_control_flow() {
        let program = parse("1 IF { 2 } ELSE { 3 } 0 PROPHECY");
        let mut encoder = SmtEncoder::new();
        let smt = encoder.encode(&program);
        
        assert!(smt.contains("ite"), "SMT output missing 'ite': {}", smt);
    }
     
    #[test]
    fn test_div_by_zero() {
        let program = parse("10 0 DIV 0 PROPHECY");
         let result = TimeLoop::new(default_config()).run(&program);
         if let ConvergenceStatus::Consistent { memory, .. } = result {
             assert_eq!(memory.read(0).val, 0);
         } else {
             panic!("Expected consistency");
         }
    }

    #[test]
    fn test_new_opcodes() {
        let program = parse("1 2 3 ROT DEPTH 0 PROPHECY");
        let result = TimeLoop::new(default_config()).run(&program);
         if let ConvergenceStatus::Consistent { memory, .. } = result {
             assert_eq!(memory.read(0).val, 3);
         } else {
             panic!("Expected consistency");
         }
    }

    #[test]
    fn test_grandfather_paradox_diagnosis() {
        let program = parse("0 ORACLE NOT 0 PROPHECY");
        let config = Config { 
            mode: ExecutionMode::Diagnostic, 
            max_epochs: 100, 
            seed: 0,
            verbose: false,
        };
        let result = TimeLoop::new(config).run(&program);
        
        // Diagnostic mode in new timeloop returns Oscillation with Diagnosis
        if let ConvergenceStatus::Oscillation { diagnosis, .. } = result {
             match diagnosis {
                 crate::timeloop::ParadoxDiagnosis::NegativeLoop { .. } => {
                     // Pass
                 },
                 _ => panic!("Expected NegativeLoop diagnosis"),
             }
        } else {
             panic!("Expected Oscillation with diagnosis, got {:?}", result);
        }
    }
}
