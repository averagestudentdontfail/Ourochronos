//! Lexer and Parser for OUROCHRONOS.
//!
//! Syntax:
//! - Numbers: 42, 0xFF (hex), 0b1010 (binary)
//! - Opcodes: ADD, SUB, ORACLE, PROPHECY, etc.
//! - Control flow: IF { } ELSE { }, WHILE { cond } { body }
//! - Comments: # line comment

use crate::ast::{OpCode, Stmt, Program};
use crate::core_types::Value;
use std::iter::Peekable;
use std::slice::Iter;

/// Tokens produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A word (identifier or keyword).
    Word(String),
    /// A numeric literal.
    Number(u64),
    /// Left brace: {
    LBrace,
    /// Right brace: }
    RBrace,
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
            
            // Symbolic operators (single character words)
            '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '~' |
            '<' | '>' | '=' | '!' => {
                // Check for two-character operators
                let mut op = String::new();
                op.push(chars[i]);
                i += 1;
                
                if i < chars.len() {
                    match (op.chars().next().unwrap(), chars[i]) {
                        ('<', '=') | ('>', '=') | ('=', '=') | ('!', '=') |
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

/// Parser for OUROCHRONOS programs.
pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    /// Create a new parser from a token slice.
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: tokens.iter().peekable(),
        }
    }
    
    /// Parse a complete program.
    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut stmts = Vec::new();
        while self.tokens.peek().is_some() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(Program { body: stmts })
    }
    
    /// Parse a single statement.
    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.tokens.next() {
            Some(Token::Number(n)) => Ok(Stmt::Push(Value::new(*n))),
            
            Some(Token::Word(w)) => self.parse_word(w),
            
            Some(Token::LBrace) => {
                let content = self.parse_block_content()?;
                Ok(Stmt::Block(content))
            }
            
            Some(Token::RBrace) => Err("Unexpected '}'".to_string()),
            
            None => Err("Unexpected end of input".to_string()),
        }
    }
    
    /// Parse a word (opcode or keyword).
    fn parse_word(&mut self, word: &str) -> Result<Stmt, String> {
        let upper = word.to_uppercase();
        
        match upper.as_str() {
            // Stack operations
            "NOP" => Ok(Stmt::Op(OpCode::Nop)),
            "HALT" => Ok(Stmt::Op(OpCode::Halt)),
            "POP" | "DROP" => Ok(Stmt::Op(OpCode::Pop)),
            "DUP" => Ok(Stmt::Op(OpCode::Dup)),
            "SWAP" => Ok(Stmt::Op(OpCode::Swap)),
            "OVER" => Ok(Stmt::Op(OpCode::Over)),
            "ROT" => Ok(Stmt::Op(OpCode::Rot)),
            "DEPTH" => Ok(Stmt::Op(OpCode::Depth)),
            
            // Arithmetic
            "ADD" | "+" => Ok(Stmt::Op(OpCode::Add)),
            "SUB" | "-" => Ok(Stmt::Op(OpCode::Sub)),
            "MUL" | "*" => Ok(Stmt::Op(OpCode::Mul)),
            "DIV" | "/" => Ok(Stmt::Op(OpCode::Div)),
            "MOD" | "%" => Ok(Stmt::Op(OpCode::Mod)),
            "NEG" => Ok(Stmt::Op(OpCode::Neg)),
            
            // Bitwise
            "NOT" | "~" => Ok(Stmt::Op(OpCode::Not)),
            "AND" | "&" => Ok(Stmt::Op(OpCode::And)),
            "OR" | "|" => Ok(Stmt::Op(OpCode::Or)),
            "XOR" | "^" => Ok(Stmt::Op(OpCode::Xor)),
            "SHL" | "<<" => Ok(Stmt::Op(OpCode::Shl)),
            "SHR" | ">>" => Ok(Stmt::Op(OpCode::Shr)),
            
            // Comparison
            "EQ" | "==" => Ok(Stmt::Op(OpCode::Eq)),
            "NEQ" | "!=" => Ok(Stmt::Op(OpCode::Neq)),
            "LT" | "<" => Ok(Stmt::Op(OpCode::Lt)),
            "GT" | ">" => Ok(Stmt::Op(OpCode::Gt)),
            "LTE" | "<=" => Ok(Stmt::Op(OpCode::Lte)),
            "GTE" | ">=" => Ok(Stmt::Op(OpCode::Gte)),
            
            // Temporal
            "ORACLE" | "READ" => Ok(Stmt::Op(OpCode::Oracle)),
            "PROPHECY" | "WRITE" => Ok(Stmt::Op(OpCode::Prophecy)),
            "PRESENT" => Ok(Stmt::Op(OpCode::PresentRead)),
            "PARADOX" => Ok(Stmt::Op(OpCode::Paradox)),
            
            // I/O
            "INPUT" => Ok(Stmt::Op(OpCode::Input)),
            "OUTPUT" => Ok(Stmt::Op(OpCode::Output)),
            
            // Control flow
            "IF" => {
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
            
            _ => Err(format!("Unknown word: {}", word)),
        }
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
}
