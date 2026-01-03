//! Language Server Protocol support for OUROCHRONOS.
//!
//! Provides IDE integration with diagnostics, completion, and hover.
//! Requires the `lsp` feature.

use std::collections::HashMap;

use crate::ast::{Program, Stmt, OpCode};
use crate::parser::Parser;

/// LSP diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// A diagnostic message.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Line number (0-indexed).
    pub line: usize,
    /// Column number (0-indexed).
    pub column: usize,
    /// End line.
    pub end_line: usize,
    /// End column.
    pub end_column: usize,
    /// Message.
    pub message: String,
    /// Severity.
    pub severity: Severity,
}

/// Completion item kind.
#[derive(Debug, Clone, Copy)]
pub enum CompletionKind {
    Keyword,
    Opcode,
    Procedure,
    Variable,
}

/// A completion suggestion.
#[derive(Debug, Clone)]
pub struct CompletionItem {
    /// Label shown.
    pub label: String,
    /// Kind.
    pub kind: CompletionKind,
    /// Detail text.
    pub detail: Option<String>,
    /// Insert text.
    pub insert_text: Option<String>,
}

/// Hover information.
#[derive(Debug, Clone)]
pub struct HoverInfo {
    /// Content in markdown.
    pub contents: String,
}

/// OUROCHRONOS language analyzer for LSP.
#[derive(Debug, Default)]
pub struct LanguageAnalyzer {
    /// Cached diagnostics per file.
    diagnostics: HashMap<String, Vec<Diagnostic>>,
    /// Known procedures.
    procedures: Vec<String>,
}

impl LanguageAnalyzer {
    /// Create a new analyzer.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Analyze source code and return diagnostics.
    pub fn analyze(&mut self, uri: &str, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        // Try to parse
        let tokens = crate::parser::tokenize(source);
        let mut parser = Parser::new(&tokens);
        match parser.parse_program() {
            Ok(program) => {
                // Type check for temporal issues
                diagnostics.extend(self.check_temporal_safety(&program));
                // Collect procedures
                self.procedures = program.procedures.iter()
                    .map(|p| p.name.clone())
                    .collect();
            }
            Err(e) => {
                diagnostics.push(Diagnostic {
                    line: 0,
                    column: 0,
                    end_line: 0,
                    end_column: 0,
                    message: format!("Parse error: {}", e),
                    severity: Severity::Error,
                });
            }
        }
        
        self.diagnostics.insert(uri.to_string(), diagnostics.clone());
        diagnostics
    }
    
    /// Check for temporal safety issues.
    fn check_temporal_safety(&self, program: &Program) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut line = 0;
        
        for stmt in &program.body {
            self.check_stmt_temporal(stmt, &mut line, &mut diagnostics);
        }
        
        diagnostics
    }
    
    fn check_stmt_temporal(&self, stmt: &Stmt, line: &mut usize, diagnostics: &mut Vec<Diagnostic>) {
        match stmt {
            Stmt::Op(OpCode::Paradox) => {
                diagnostics.push(Diagnostic {
                    line: *line,
                    column: 0,
                    end_line: *line,
                    end_column: 7,
                    message: "PARADOX will abort execution".to_string(),
                    severity: Severity::Warning,
                });
            }
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.check_stmt_temporal(s, line, diagnostics);
                }
            }
            Stmt::If { then_branch, else_branch } => {
                for s in then_branch {
                    self.check_stmt_temporal(s, line, diagnostics);
                }
                if let Some(eb) = else_branch {
                    for s in eb {
                        self.check_stmt_temporal(s, line, diagnostics);
                    }
                }
            }
            _ => {}
        }
        *line += 1;
    }
    
    /// Get completions at a position.
    pub fn get_completions(&self, _line: usize, _column: usize) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        
        // Keywords
        for kw in &["IF", "ELSE", "WHILE", "PROCEDURE", "LET", "MODULE", "IMPORT", "EXPORT"] {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: CompletionKind::Keyword,
                detail: Some("keyword".to_string()),
                insert_text: None,
            });
        }
        
        // Opcodes
        for op in &[
            ("ADD", "Pop two values, push sum"),
            ("SUB", "Pop two values, push difference"),
            ("MUL", "Pop two values, push product"),
            ("DIV", "Pop two values, push quotient"),
            ("MOD", "Pop two values, push remainder"),
            ("DUP", "Duplicate top of stack"),
            ("POP", "Remove top of stack"),
            ("SWAP", "Swap top two values"),
            ("ORACLE", "Read from past (anamnesis)"),
            ("PROPHECY", "Write to future (present)"),
            ("PRESENT_READ", "Read from current present"),
            ("PARADOX", "Signal temporal paradox"),
            ("HALT", "Stop execution"),
            ("INPUT", "Read from stdin"),
            ("OUTPUT", "Write to stdout"),
        ] {
            items.push(CompletionItem {
                label: op.0.to_string(),
                kind: CompletionKind::Opcode,
                detail: Some(op.1.to_string()),
                insert_text: None,
            });
        }
        
        // Known procedures
        for proc in &self.procedures {
            items.push(CompletionItem {
                label: proc.clone(),
                kind: CompletionKind::Procedure,
                detail: Some("procedure".to_string()),
                insert_text: Some(format!("{}();", proc)),
            });
        }
        
        items
    }
    
    /// Get hover information.
    pub fn get_hover(&self, word: &str) -> Option<HoverInfo> {
        let info = match word.to_uppercase().as_str() {
            "ORACLE" => "**ORACLE** `addr`\n\nReads a value from the past (anamnesis memory).\nPops address, pushes value from that address.",
            "PROPHECY" => "**PROPHECY** `addr val`\n\nWrites a value to the future (present memory).\nPops address and value, writes value at address.",
            "PARADOX" => "**PARADOX**\n\nSignals a temporal paradox.\nAborts the current epoch with paradox status.",
            "HALT" => "**HALT**\n\nStops epoch execution normally.",
            "IF" => "**IF** `{ ... }` **ELSE** `{ ... }`\n\nConditional branching.\nPops condition, executes then-branch if non-zero.",
            "WHILE" => "**WHILE** `{ cond }` `{ body }`\n\nLoop while condition is non-zero.",
            "PROCEDURE" => "**PROCEDURE** `name` `{ body }`\n\nDefines a reusable procedure.\nCall with `name();`",
            "LET" => "**LET** `name = expr;`\n\nBinds the result of expression to a name.\nDesugars to PICK operation.",
            _ => return None,
        };
        
        Some(HoverInfo {
            contents: info.to_string(),
        })
    }
}

#[cfg(feature = "lsp")]
pub mod server {
    //! LSP server implementation using lsp-server crate.
    
    use lsp_server::{Connection, Message, RequestId, Response};
    use lsp_types::*;
    use super::*;
    
    /// Run the LSP server.
    pub fn run_server() -> Result<(), Box<dyn std::error::Error>> {
        let (connection, io_threads) = Connection::stdio();
        
        // Initialize
        let caps = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::FULL
            )),
            completion_provider: Some(CompletionOptions::default()),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            ..Default::default()
        };
        
        let init_result = serde_json::to_value(InitializeResult {
            capabilities: caps,
            server_info: Some(ServerInfo {
                name: "ourochronos-lsp".to_string(),
                version: Some("0.2.0".to_string()),
            }),
        })?;
        
        connection.initialize(init_result)?;
        
        let mut analyzer = LanguageAnalyzer::new();
        
        // Main loop
        for msg in &connection.receiver {
            match msg {
                Message::Request(req) => {
                    if connection.handle_shutdown(&req)? {
                        break;
                    }
                    // Handle other requests...
                }
                Message::Notification(notif) => {
                    // Handle notifications like textDocument/didOpen, didChange
                    if notif.method == "textDocument/didOpen" {
                        if let Ok(params) = serde_json::from_value::<DidOpenTextDocumentParams>(notif.params) {
                            let uri = params.text_document.uri.to_string();
                            let text = params.text_document.text;
                            let _diags = analyzer.analyze(&uri, &text);
                            // Would send diagnostics back...
                        }
                    }
                }
                Message::Response(_) => {}
            }
        }
        
        io_threads.join()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_analyzer_creation() {
        let analyzer = LanguageAnalyzer::new();
        assert!(analyzer.procedures.is_empty());
    }
    
    #[test]
    fn test_completions() {
        let analyzer = LanguageAnalyzer::new();
        let completions = analyzer.get_completions(0, 0);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "ORACLE"));
    }
    
    #[test]
    fn test_hover() {
        let analyzer = LanguageAnalyzer::new();
        let hover = analyzer.get_hover("ORACLE");
        assert!(hover.is_some());
        assert!(hover.unwrap().contents.contains("anamnesis"));
    }
    
    #[test]
    fn test_analyze_valid() {
        let mut analyzer = LanguageAnalyzer::new();
        let source = "1 2 ADD";
        let diags = analyzer.analyze("test.ouro", source);
        assert!(diags.is_empty());
    }
}
