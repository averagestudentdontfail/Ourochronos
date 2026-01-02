use crate::ast::{OpCode, Stmt, Program};
use crate::core_types::{Value, Address};
use std::iter::Peekable;
use std::slice::Iter;

/// Tokens for the parser.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Number(u64),
    LBrace, // {
    RBrace, // }
    Semi,   // ;
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\n' | '\r' => {
                i += 1;
            }
            '{' => {
                tokens.push(Token::LBrace);
                i += 1;
            }
            '}' => {
                tokens.push(Token::RBrace);
                i += 1;
            }
            ';' => {
                tokens.push(Token::Semi);
                i += 1;
            }
            '#' => {
                // Comment until newline
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            c if c.is_digit(10) => {
                let mut num_str = String::new();
                while i < chars.len() && chars[i].is_digit(10) {
                    num_str.push(chars[i]);
                    i += 1;
                }
                if let Ok(n) = num_str.parse::<u64>() {
                    tokens.push(Token::Number(n));
                }
            }
            c if !c.is_whitespace() => {
                let mut word = String::new();
                while i < chars.len() && !chars[i].is_whitespace() && chars[i] != ';' && chars[i] != '{' && chars[i] != '}' {
                    word.push(chars[i]);
                    i += 1;
                }
                tokens.push(Token::Word(word));
            }
            _ => i += 1,
        }
    }
    tokens
}

pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: tokens.iter().peekable(),
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut instrs = Vec::new();
        while self.tokens.peek().is_some() {
            instrs.push(self.parse_stmt()?);
        }
        Ok(Program { body: instrs })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.tokens.next() {
            Some(Token::Number(n)) => Ok(Stmt::Push(Value::new(*n))),
            Some(Token::Word(w)) => match w.as_str() {
                // Stack Ops
                "nop" | "NOP" => Ok(Stmt::Op(OpCode::Nop)),
                "pop" | "POP" => Ok(Stmt::Op(OpCode::Pop)),
                "dup" | "DUP" => Ok(Stmt::Op(OpCode::Dup)),
                "swap" | "SWAP" => Ok(Stmt::Op(OpCode::Swap)),
                "over" | "OVER" => Ok(Stmt::Op(OpCode::Over)),
                
                // Arithmetic
                "add" | "ADD" | "+" => Ok(Stmt::Op(OpCode::Add)),
                "sub" | "SUB" | "-" => Ok(Stmt::Op(OpCode::Sub)),
                "mul" | "MUL" | "*" => Ok(Stmt::Op(OpCode::Mul)),
                "div" | "DIV" | "/" => Ok(Stmt::Op(OpCode::Div)),
                "mod" | "MOD" | "%" => Ok(Stmt::Op(OpCode::Mod)),
                
                // Logic
                "not" | "NOT" | "!" => Ok(Stmt::Op(OpCode::Not)),
                "and" | "AND" | "&" => Ok(Stmt::Op(OpCode::And)),
                "or" | "OR" | "|" => Ok(Stmt::Op(OpCode::Or)),
                "xor" | "XOR" | "^" => Ok(Stmt::Op(OpCode::Xor)),
                
                // Compare
                "eq" | "EQ" | "==" => Ok(Stmt::Op(OpCode::Eq)),
                "neq" | "NEQ" | "!=" => Ok(Stmt::Op(OpCode::Neq)),
                "gt" | "GT" | ">" => Ok(Stmt::Op(OpCode::Gt)),
                "lt" | "LT" | "<" => Ok(Stmt::Op(OpCode::Lt)),

                // Temporal
                "oracle" | "ORACLE" | "READ" => Ok(Stmt::Op(OpCode::Oracle)),
                "prophecy" | "PROPHECY" | "WRITE" => Ok(Stmt::Op(OpCode::Prophecy)),
                "paradox" | "PARADOX" => Ok(Stmt::Op(OpCode::Paradox)),
                
                // I/O
                "input" | "INPUT" => Ok(Stmt::Op(OpCode::Input)),
                "output" | "OUTPUT" => Ok(Stmt::Op(OpCode::Output)),
                
                // Control Flow
                "if" | "IF" => {
                    let then_block = self.parse_block()?;
                    let mut else_block = None;
                    if let Some(Token::Word(w)) = self.tokens.peek() {
                        if w == "else" || w == "ELSE" {
                            self.tokens.next(); // consume else
                            else_block = Some(self.parse_block()?);
                        }
                    }
                    Ok(Stmt::If { then_branch: then_block, else_branch: else_block })
                }
                "while" | "WHILE" => {
                    // Logic: while { cond } { body }
                    
                    let cond_block = self.parse_block()?;
                    // Expect another block
                    let body_block = self.parse_block()?;
                    Ok(Stmt::While { cond: cond_block, body: body_block })
                }
                
                _ => Err(format!("Unknown word: {}", w)),
            },
            Some(Token::LBrace) => {
                // Anonymous block? or just syntax error if not inside IF/WHILE
                // But parse_block uses { ... }
                // So if we see { here, it's a block.
                // We need to put it back? 
                // Wait, parse_block EXPECTS { as first token.
                // But here we consumed it.
                // So we can call parse_block_content.
                let content = self.parse_block_content()?;
                Ok(Stmt::Block(content))
            }
            Some(Token::RBrace) => Err("Unexpected }".to_string()),
            Some(Token::Semi) => Ok(Stmt::Op(OpCode::Nop)), // Empty stmt
            None => Err("Unexpected EOF".to_string()),
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        if let Some(Token::LBrace) = self.tokens.next() {
            self.parse_block_content()
        } else {
            Err("Expected { for block".to_string())
        }
    }
    
    fn parse_block_content(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while let Some(token) = self.tokens.peek() {
            if let Token::RBrace = token {
                self.tokens.next(); // consume }
                return Ok(stmts);
            }
            stmts.push(self.parse_stmt()?);
        }
        Err("Unclosed block, expected }".to_string())
    }
}
