//! Lexer and Parser for OUROCHRONOS.
//!
//! Syntax:
//! - Numbers: 42, 0xFF (hex), 0b1010 (binary)
//! - Opcodes: ADD, SUB, ORACLE, PROPHECY, etc.
//! - Control flow: IF { } ELSE { }, WHILE { cond } { body }
//! - Comments: # line comment

use crate::ast::{OpCode, Stmt, Program, Procedure};
use crate::core_types::Value;
use std::iter::Peekable;
use std::slice::Iter;
use std::collections::HashMap;

/// Tokens produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A word (identifier or keyword).
    Word(String),
    /// A numeric literal.
    Number(u64),
    /// A string literal.
    StringLit(String),
    /// A character literal.
    CharLit(char),
    /// Left brace: {
    LBrace,
    /// Right brace: }
    RBrace,
    /// Equals sign for LET: =
    Equals,
    /// Semicolon for statement termination: ;
    Semicolon,
}

/// Tokenize source code into a sequence of tokens.
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        match chars[i] {
            // Whitespace
            ' ' | '\t' | '\n' | '\r' => {
                i += 1;
            }
            
            // Block delimiters
            '{' => {
                tokens.push(Token::LBrace);
                i += 1;
            }
            '}' => {
                tokens.push(Token::RBrace);
                i += 1;
            }
            
            // Line comment
            '#' => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            
            // String literal "..."
            '"' => {
                i += 1; // Skip opening quote
                let mut string_val = String::new();
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        // Escape sequence
                        i += 1;
                        match chars[i] {
                            'n' => string_val.push('\n'),
                            't' => string_val.push('\t'),
                            'r' => string_val.push('\r'),
                            '"' => string_val.push('"'),
                            '\\' => string_val.push('\\'),
                            c => string_val.push(c),
                        }
                    } else {
                        string_val.push(chars[i]);
                    }
                    i += 1;
                }
                if i < chars.len() {
                    i += 1; // Skip closing quote
                }
                tokens.push(Token::StringLit(string_val));
            }
            
            // Character literal '...'
            '\'' => {
                i += 1; // Skip opening quote
                if i < chars.len() {
                    let ch = if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 1;
                        match chars[i] {
                            'n' => '\n',
                            't' => '\t',
                            'r' => '\r',
                            '\'' => '\'',
                            '\\' => '\\',
                            c => c,
                        }
                    } else {
                        chars[i]
                    };
                    i += 1;
                    if i < chars.len() && chars[i] == '\'' {
                        i += 1; // Skip closing quote
                    }
                    tokens.push(Token::CharLit(ch));
                }
            }
            
            // Semicolon comment (alternative)
            ';' if i + 1 < chars.len() && chars[i + 1] == ';' => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            
            // Numeric literal
            c if c.is_ascii_digit() => {
                let start = i;
                
                // Check for hex (0x) or binary (0b)
                if c == '0' && i + 1 < chars.len() {
                    match chars[i + 1] {
                        'x' | 'X' => {
                            i += 2; // Skip 0x
                            let hex_start = i;
                            while i < chars.len() && chars[i].is_ascii_hexdigit() {
                                i += 1;
                            }
                            let hex_str: String = chars[hex_start..i].iter().collect();
                            if let Ok(n) = u64::from_str_radix(&hex_str, 16) {
                                tokens.push(Token::Number(n));
                            }
                            continue;
                        }
                        'b' | 'B' => {
                            i += 2; // Skip 0b
                            let bin_start = i;
                            while i < chars.len() && (chars[i] == '0' || chars[i] == '1') {
                                i += 1;
                            }
                            let bin_str: String = chars[bin_start..i].iter().collect();
                            if let Ok(n) = u64::from_str_radix(&bin_str, 2) {
                                tokens.push(Token::Number(n));
                            }
                            continue;
                        }
                        _ => {}
                    }
                }
                
                // Decimal number
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                if let Ok(n) = num_str.parse::<u64>() {
                    tokens.push(Token::Number(n));
                }
            }
            
            // Word (identifier or keyword)
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                tokens.push(Token::Word(word));
            }
            
            // Equals sign
            '=' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                // == is a comparison operator
                tokens.push(Token::Word("==".to_string()));
                i += 2;
            }
            '=' => {
                tokens.push(Token::Equals);
                i += 1;
            }
            
            // Semicolon (statement terminator)
            ';' => {
                // Check if it's a comment (;;)
                if i + 1 < chars.len() && chars[i + 1] == ';' {
                    while i < chars.len() && chars[i] != '\n' {
                        i += 1;
                    }
                } else {
                    tokens.push(Token::Semicolon);
                    i += 1;
                }
            }
            
            // Symbolic operators (single character words)
            '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '~' |
            '<' | '>' | '!' => {
                // Check for two-character operators
                let mut op = String::new();
                op.push(chars[i]);
                i += 1;
                
                if i < chars.len() {
                    match (op.chars().next().unwrap(), chars[i]) {
                        ('<', '=') | ('>', '=') | ('!', '=') |
                        ('<', '<') | ('>', '>') => {
                            op.push(chars[i]);
                            i += 1;
                        }
                        _ => {}
                    }
                }
                
                tokens.push(Token::Word(op));
            }
            
            // Skip unknown characters
            _ => {
                i += 1;
            }
        }
    }
    
    tokens
}

/// Variable binding with its stack position.
#[derive(Debug, Clone)]
struct VariableBinding {
    /// Stack depth when this variable was created.
    stack_depth: usize,
}

/// Procedure binding with its definition.
#[derive(Debug, Clone)]
struct ProcedureBinding {
    /// Number of parameters the procedure takes.
    param_count: usize,
    /// Number of return values.
    return_count: usize,
}

/// Parser for OUROCHRONOS programs.
pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
    constants: HashMap<String, u64>,
    /// Variable bindings (name -> binding info).
    variables: HashMap<String, VariableBinding>,
    /// Procedure bindings (name -> binding info).
    procedures: HashMap<String, ProcedureBinding>,
    /// Parsed procedure definitions.
    procedure_defs: Vec<Procedure>,
    /// Current stack depth (for variable resolution).
    stack_depth: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser from a token slice.
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: tokens.iter().peekable(),
            constants: HashMap::new(),
            variables: HashMap::new(),
            procedures: HashMap::new(),
            procedure_defs: Vec::new(),
            stack_depth: 0,
        }
    }
    
    /// Emit an opcode and track its stack effect.
    fn emit_op(&mut self, op: OpCode) -> Result<Stmt, String> {
        let (inputs, outputs) = op.stack_effect();
        // Saturating sub to avoid negative depth (parser doesn't validate thoroughly)
        self.stack_depth = self.stack_depth.saturating_sub(inputs);
        self.stack_depth += outputs;
        Ok(Stmt::Op(op))
    }
    
    /// Parse a complete program.
    pub fn parse_program(&mut self) -> Result<Program, String> {
        // Parse declarations (MANIFEST) and procedures
        loop {
            if self.peek_word_eq("MANIFEST") {
                self.parse_declaration()?;
            } else if self.peek_word_eq("PROCEDURE") || self.peek_word_eq("PROC") {
                self.parse_procedure_def()?;
            } else {
                break;
            }
        }

        let mut stmts = Vec::new();
        while self.tokens.peek().is_some() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(Program { 
            procedures: std::mem::take(&mut self.procedure_defs),
            body: stmts,
        })
    }

    /// Parse a declaration: MANIFEST name = value;
    fn parse_declaration(&mut self) -> Result<(), String> {
        self.tokens.next(); // Consume MANIFEST

        let name = match self.tokens.next() {
            Some(Token::Word(w)) => w.to_uppercase(),
            _ => return Err("Expected identifier after MANIFEST".to_string()),
        };

        // Expect '='
        if !self.peek_word_eq("=") {
             // Allow omitting '=' if user prefers just MANIFEST NAME VALUE; but spec says '='
             // Let's check for symbolic '=' which is Token::Word("=")
             match self.tokens.peek() {
                 Some(Token::Word(w)) if w == "=" => { self.tokens.next(); },
                 _ => return Err("Expected '=' in declaration".to_string()),
             }
        } else {
             self.tokens.next();
        }

        // Expect value (simple integer for now)
        let value = match self.tokens.next() {
            Some(Token::Number(n)) => *n,
            _ => return Err("Expected integer value in declaration".to_string()),
        };

        // Expect ';'
        if !self.peek_word_eq(";") {
             match self.tokens.peek() {
                 Some(Token::Word(w)) if w == ";" => { self.tokens.next(); },
                 _ => return Err("Expected ';' after declaration".to_string()),
             }
        } else {
             self.tokens.next();
        }

        self.constants.insert(name, value);
        Ok(())
    }
    
    /// Parse a single statement.
    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.tokens.next() {
            Some(Token::Number(n)) => {
                self.stack_depth += 1;
                Ok(Stmt::Push(Value::new(*n)))
            }
            
            Some(Token::Word(w)) => self.parse_word(w),
            
            Some(Token::LBrace) => {
                let content = self.parse_block_content()?;
                Ok(Stmt::Block(content))
            }
            
            Some(Token::StringLit(s)) => {
                // Push each character as a value onto the stack
                // Then push the length at the end
                let chars: Vec<char> = s.chars().collect();
                let mut stmts: Vec<Stmt> = chars.iter()
                    .map(|c| {
                        self.stack_depth += 1;
                        Stmt::Push(Value::new(*c as u64))
                    })
                    .collect();
                // Push length
                self.stack_depth += 1;
                stmts.push(Stmt::Push(Value::new(chars.len() as u64)));
                Ok(Stmt::Block(stmts))
            }
            
            Some(Token::CharLit(c)) => {
                self.stack_depth += 1;
                Ok(Stmt::Push(Value::new(*c as u64)))
            }
            
            Some(Token::RBrace) => Err("Unexpected '}'".to_string()),
            Some(Token::Equals) => Err("Unexpected '='".to_string()),
            Some(Token::Semicolon) => Err("Unexpected ';'".to_string()),
            
            None => Err("Unexpected end of input".to_string()),
        }
    }
    
    /// Parse a word (opcode or keyword).
    fn parse_word(&mut self, word: &str) -> Result<Stmt, String> {
        let upper = word.to_uppercase();
        
        match upper.as_str() {
            // Stack operations
            "NOP" => self.emit_op(OpCode::Nop),
            "HALT" => self.emit_op(OpCode::Halt),
            "POP" | "DROP" => self.emit_op(OpCode::Pop),
            "DUP" => self.emit_op(OpCode::Dup),
            "SWAP" => self.emit_op(OpCode::Swap),
            "OVER" => self.emit_op(OpCode::Over),
            "ROT" => self.emit_op(OpCode::Rot),
            "DEPTH" => self.emit_op(OpCode::Depth),
            "PICK" | "PEEK" => self.emit_op(OpCode::Pick),
            
            // Arithmetic
            "ADD" | "+" => self.emit_op(OpCode::Add),
            "SUB" | "-" => self.emit_op(OpCode::Sub),
            "MUL" | "*" => self.emit_op(OpCode::Mul),
            "DIV" | "/" => self.emit_op(OpCode::Div),
            "MOD" | "%" => self.emit_op(OpCode::Mod),
            "NEG" => self.emit_op(OpCode::Neg),
            
            // Bitwise
            "NOT" | "~" => self.emit_op(OpCode::Not),
            "AND" | "&" => self.emit_op(OpCode::And),
            "OR" | "|" => self.emit_op(OpCode::Or),
            "XOR" | "^" => self.emit_op(OpCode::Xor),
            "SHL" | "<<" => self.emit_op(OpCode::Shl),
            "SHR" | ">>" => self.emit_op(OpCode::Shr),
            
            // Comparison
            "EQ" | "==" => self.emit_op(OpCode::Eq),
            "NEQ" | "!=" => self.emit_op(OpCode::Neq),
            "LT" | "<" => self.emit_op(OpCode::Lt),
            "GT" | ">" => self.emit_op(OpCode::Gt),
            "LTE" | "<=" => self.emit_op(OpCode::Lte),
            "GTE" | ">=" => self.emit_op(OpCode::Gte),
            
            // Temporal
            "ORACLE" | "READ" => self.emit_op(OpCode::Oracle),
            "PROPHECY" | "WRITE" => self.emit_op(OpCode::Prophecy),
            "PRESENT" => self.emit_op(OpCode::PresentRead),
            "PARADOX" => self.emit_op(OpCode::Paradox),
            
            // I/O
            "INPUT" => self.emit_op(OpCode::Input),
            "OUTPUT" => self.emit_op(OpCode::Output),
            
            // Control flow
            "IF" => {
                // Pop condition
                self.stack_depth = self.stack_depth.saturating_sub(1);
                let then_block = self.parse_block()?;
                let else_block = if self.peek_word_eq("ELSE") {
                    self.tokens.next(); // consume ELSE
                    Some(self.parse_block()?)
                } else {
                    None
                };
                Ok(Stmt::If { then_branch: then_block, else_branch: else_block })
            }
            
            "WHILE" => {
                let cond_block = self.parse_block()?;
                let body_block = self.parse_block()?;
                Ok(Stmt::While { cond: cond_block, body: body_block })
            }
            
            "ELSE" => Err("Unexpected ELSE without IF".to_string()),
            
            // LET bindings: LET name = expr;
            "LET" => self.parse_let(),
            
            // Handle constants, variables, or unknown words
            other => {
                // Check if it is a defined constant
                if let Some(&val) = self.constants.get(other) {
                    self.stack_depth += 1;
                    return Ok(Stmt::Push(Value::new(val)));
                }
                
                // Check if it is a defined variable (case-insensitive)
                let lower = word.to_lowercase();
                if let Some(binding) = self.variables.get(&lower).cloned() {
                    // Calculate PICK index: distance from current stack top to variable position
                    let pick_index = self.stack_depth - binding.stack_depth - 1;
                    self.stack_depth += 1; // PICK adds to stack
                    // Generate: <pick_index> PICK
                    return Ok(Stmt::Block(vec![
                        Stmt::Push(Value::new(pick_index as u64)),
                        Stmt::Op(OpCode::Pick),
                    ]));
                }
                
                // Check if it is a defined procedure
                if let Some(binding) = self.procedures.get(&lower).cloned() {
                    // Procedure call: consumes params, produces returns
                    self.stack_depth = self.stack_depth.saturating_sub(binding.param_count);
                    self.stack_depth += binding.return_count;
                    return Ok(Stmt::Call { name: lower });
                }
                
                // If it is strictly uppercase, it might be a misspelled opcode
                if other.chars().all(|c| c.is_uppercase() || c == '_') {
                     return Err(format!("Unknown opcode or constant: {}", other));
                }
                // Otherwise, error.
                Err(format!("Unknown variable or procedure: {}", other))
            }
        }
    }
    
    /// Parse a LET binding: LET name = expr;
    fn parse_let(&mut self) -> Result<Stmt, String> {
        // Get variable name
        let name = match self.tokens.next() {
            Some(Token::Word(w)) => w.to_lowercase(),
            _ => return Err("Expected variable name after LET".to_string()),
        };
        
        // Expect '='
        match self.tokens.next() {
            Some(Token::Equals) => {}
            Some(Token::Word(w)) if w == "=" => {}
            _ => return Err("Expected '=' after variable name in LET".to_string()),
        }
        
        // Parse the expression (collect statements until semicolon)
        let mut expr_stmts = Vec::new();
        loop {
            match self.tokens.peek() {
                Some(Token::Semicolon) => {
                    self.tokens.next(); // consume ;
                    break;
                }
                Some(Token::RBrace) | None => {
                    // End of block or input - allow missing semicolon
                    break;
                }
                _ => {
                    expr_stmts.push(self.parse_stmt()?);
                }
            }
        }
        
        // Register the variable at current stack depth
        // (the expression left its result on the stack)
        self.variables.insert(name, VariableBinding {
            stack_depth: self.stack_depth - 1,
        });
        
        // Return the expression statements as a block
        if expr_stmts.len() == 1 {
            Ok(expr_stmts.remove(0))
        } else {
            Ok(Stmt::Block(expr_stmts))
        }
    }
    
    /// Parse a procedure definition: PROCEDURE name(params) { body }
    fn parse_procedure_def(&mut self) -> Result<(), String> {
        self.tokens.next(); // Consume PROCEDURE
        
        // Get procedure name
        let name = match self.tokens.next() {
            Some(Token::Word(w)) => w.to_lowercase(),
            _ => return Err("Expected procedure name after PROCEDURE".to_string()),
        };
        
        // Parse optional parameter list
        let mut params = Vec::new();
        if self.peek_word_eq("(") || matches!(self.tokens.peek(), Some(Token::Word(w)) if w == "(") {
            self.tokens.next(); // consume (
            loop {
                match self.tokens.peek() {
                    Some(Token::Word(w)) if w == ")" => {
                        self.tokens.next();
                        break;
                    }
                    Some(Token::Word(w)) if w == "," => {
                        self.tokens.next();
                    }
                    Some(Token::Word(w)) => {
                        params.push(w.to_lowercase());
                        self.tokens.next();
                    }
                    _ => break,
                }
            }
        }
        
        // Parse body block
        let body = self.parse_block()?;
        
        // Register procedure
        let param_count = params.len();
        self.procedures.insert(name.clone(), ProcedureBinding {
            param_count,
            return_count: 1, // Default: procedures return 1 value
        });
        
        self.procedure_defs.push(Procedure {
            name,
            params,
            returns: 1,
            body,
        });
        
        Ok(())
    }
    
    /// Parse a block enclosed in braces.
    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        match self.tokens.next() {
            Some(Token::LBrace) => self.parse_block_content(),
            _ => Err("Expected '{'".to_string()),
        }
    }
    
    /// Parse the content of a block (after '{').
    fn parse_block_content(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        
        loop {
            match self.tokens.peek() {
                Some(Token::RBrace) => {
                    self.tokens.next(); // consume '}'
                    return Ok(stmts);
                }
                None => return Err("Unclosed block, expected '}'".to_string()),
                _ => stmts.push(self.parse_stmt()?),
            }
        }
    }
    
    /// Check if the next token is a word equal to the given string.
    fn peek_word_eq(&mut self, expected: &str) -> bool {
        match self.tokens.peek() {
            Some(Token::Word(w)) => w.to_uppercase() == expected,
            _ => false,
        }
    }
}

/// Parse source code directly into a program.
pub fn parse(source: &str) -> Result<Program, String> {
    let tokens = tokenize(source);
    let mut parser = Parser::new(&tokens);
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenize_numbers() {
        let tokens = tokenize("42 0xFF 0b1010");
        assert_eq!(tokens, vec![
            Token::Number(42),
            Token::Number(255),
            Token::Number(10),
        ]);
    }
    
    #[test]
    fn test_tokenize_words() {
        let tokens = tokenize("ADD SUB ORACLE");
        assert_eq!(tokens, vec![
            Token::Word("ADD".to_string()),
            Token::Word("SUB".to_string()),
            Token::Word("ORACLE".to_string()),
        ]);
    }
    
    #[test]
    fn test_tokenize_comments() {
        let tokens = tokenize("ADD # this is a comment\nSUB");
        assert_eq!(tokens, vec![
            Token::Word("ADD".to_string()),
            Token::Word("SUB".to_string()),
        ]);
    }
    
    #[test]
    fn test_parse_simple() {
        let program = parse("10 20 ADD OUTPUT").unwrap();
        assert_eq!(program.body.len(), 4);
    }
    
    #[test]
    fn test_parse_if() {
        let program = parse("1 IF { 42 OUTPUT } ELSE { 0 OUTPUT }").unwrap();
        match &program.body[1] {
            Stmt::If { then_branch, else_branch } => {
                assert_eq!(then_branch.len(), 2);
                assert!(else_branch.is_some());
            }
            _ => panic!("Expected If statement"),
        }
    }
    
    #[test]
    fn test_trivially_consistent() {
        let program = parse("10 20 ADD OUTPUT").unwrap();
        assert!(program.is_trivially_consistent());
        
        let temporal = parse("0 ORACLE 0 PROPHECY").unwrap();
        assert!(!temporal.is_trivially_consistent());
    }
    
    #[test]
    fn test_tokenize_let() {
        let tokens = tokenize("LET x = 42;");
        assert_eq!(tokens, vec![
            Token::Word("LET".to_string()),
            Token::Word("x".to_string()),
            Token::Equals,
            Token::Number(42),
            Token::Semicolon,
        ]);
    }
    
    #[test]
    fn test_parse_let_simple() {
        // LET x = 42; should parse to Push(42)
        let program = parse("LET x = 42;").unwrap();
        assert_eq!(program.body.len(), 1);
        match &program.body[0] {
            Stmt::Push(v) => assert_eq!(v.val, 42),
            other => panic!("Expected Push, got {:?}", other),
        }
    }
    
    #[test]
    fn test_parse_let_with_reference() {
        // LET x = 10; x should generate: Push(10), then a PICK to get x
        let program = parse("LET x = 10; x OUTPUT").unwrap();
        // Should have: Push(10), Block([Push(0), Pick]), Output
        assert_eq!(program.body.len(), 3);
        
        // First should be Push(10)
        match &program.body[0] {
            Stmt::Push(v) => assert_eq!(v.val, 10),
            _ => panic!("Expected Push for LET"),
        }
        
        // Second should be Block with PICK
        match &program.body[1] {
            Stmt::Block(stmts) => {
                assert_eq!(stmts.len(), 2);
                match &stmts[1] {
                    Stmt::Op(OpCode::Pick) => {}
                    _ => panic!("Expected PICK in variable reference"),
                }
            }
            _ => panic!("Expected Block for variable reference"),
        }
    }
    
    #[test]
    fn test_parse_let_expression() {
        // LET x = 10 20 ADD;
        let program = parse("LET x = 10 20 ADD;").unwrap();
        // Should have one Block containing Push(10), Push(20), Add
        match &program.body[0] {
            Stmt::Block(stmts) => {
                assert_eq!(stmts.len(), 3);
            }
            _ => panic!("Expected Block for multi-statement LET expression"),
        }
    }
}
