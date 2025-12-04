//! Fusabi Language Server Protocol Implementation
//!
//! Provides IDE features for Fusabi: diagnostics, hover, and completion.

use std::collections::HashMap;
use std::sync::RwLock;

use fusabi_frontend::{Lexer, Parser};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

pub struct FusabiLanguageServer {
    client: Client,
    documents: RwLock<HashMap<Url, String>>,
}

impl FusabiLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: RwLock::new(HashMap::new()),
        }
    }

    async fn publish_diagnostics(&self, uri: Url, text: &str) {
        let diagnostics = self.analyze(text);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    fn analyze(&self, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let mut lexer = Lexer::new(text);
        let tokens = match lexer.tokenize() {
            Ok(tokens) => tokens,
            Err(e) => {
                let (line, col, msg) = match &e {
                    fusabi_frontend::LexError::UnexpectedChar(ch, pos) => {
                        (pos.line, pos.column, format!("Unexpected character: '{}'", ch))
                    }
                    fusabi_frontend::LexError::UnterminatedString(pos) => {
                        (pos.line, pos.column, "Unterminated string literal".to_string())
                    }
                    fusabi_frontend::LexError::InvalidNumber(s, pos) => {
                        (pos.line, pos.column, format!("Invalid number: '{}'", s))
                    }
                    fusabi_frontend::LexError::UnterminatedComment(pos) => {
                        (pos.line, pos.column, "Unterminated comment".to_string())
                    }
                };
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: (line.saturating_sub(1)) as u32,
                            character: (col.saturating_sub(1)) as u32,
                        },
                        end: Position {
                            line: (line.saturating_sub(1)) as u32,
                            character: col as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("fusabi".to_string()),
                    message: msg,
                    ..Default::default()
                });
                return diagnostics;
            }
        };

        let mut parser = Parser::new(tokens);
        if let Err(e) = parser.parse_program() {
            let (line, col, msg) = match &e {
                fusabi_frontend::ParseError::UnexpectedToken {
                    expected,
                    found,
                    pos,
                } => (
                    pos.line,
                    pos.column,
                    format!("Expected {}, found {}", expected, found),
                ),
                fusabi_frontend::ParseError::UnexpectedEof { expected } => {
                    let lines: Vec<&str> = text.lines().collect();
                    let last_line = lines.len().max(1);
                    let last_col = lines.last().map(|l| l.len()).unwrap_or(0);
                    (last_line, last_col, format!("Unexpected end of file, expected {}", expected))
                }
                fusabi_frontend::ParseError::InvalidExpr { message, pos } => {
                    (pos.line, pos.column, message.clone())
                }
            };
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: (line.saturating_sub(1)) as u32,
                        character: (col.saturating_sub(1)) as u32,
                    },
                    end: Position {
                        line: (line.saturating_sub(1)) as u32,
                        character: col as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("fusabi".to_string()),
                message: msg,
                ..Default::default()
            });
        }

        diagnostics
    }

    fn get_hover_info(&self, text: &str, position: Position) -> Option<String> {
        let lines: Vec<&str> = text.lines().collect();
        let line = lines.get(position.line as usize)?;
        
        let char_pos = position.character as usize;
        if char_pos >= line.len() {
            return None;
        }

        let start = line[..char_pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let end = line[char_pos..]
            .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '!')
            .map(|i| i + char_pos)
            .unwrap_or(line.len());

        let word = &line[start..end];
        if word.is_empty() {
            return None;
        }

        self.get_keyword_docs(word)
    }

    fn get_keyword_docs(&self, word: &str) -> Option<String> {
        let docs = match word {
            "let" => "**let** - Bind a value to a name\n\n```fusabi\nlet x = 42\nlet add a b = a + b\n```",
            "let!" => "**let!** - Async/computation expression binding\n\n```fusabi\nasync { let! result = fetchData() }\n```",
            "rec" => "**rec** - Recursive binding\n\n```fusabi\nlet rec factorial n = if n <= 1 then 1 else n * factorial (n - 1)\n```",
            "in" => "**in** - Body of let expression\n\n```fusabi\nlet x = 5 in x + 1\n```",
            "if" => "**if** - Conditional expression\n\n```fusabi\nif condition then trueValue else falseValue\n```",
            "then" => "**then** - True branch of if expression",
            "else" => "**else** - False branch of if expression",
            "fun" => "**fun** - Lambda/anonymous function\n\n```fusabi\nfun x -> x + 1\nfun x y -> x + y\n```",
            "match" => "**match** - Pattern matching\n\n```fusabi\nmatch value with\n| Some x -> x\n| None -> 0\n```",
            "with" => "**with** - Match arms or record update\n\n```fusabi\n{ record with field = newValue }\n```",
            "type" => "**type** - Type definition\n\n```fusabi\ntype Option = Some of int | None\ntype Person = { name: string; age: int }\n```",
            "of" => "**of** - Discriminated union variant payload",
            "module" => "**module** - Module definition\n\n```fusabi\nmodule Math =\n  let pi = 3.14159\n```",
            "open" => "**open** - Import module\n\n```fusabi\nopen Math\n```",
            "async" => "**async** - Async computation expression\n\n```fusabi\nasync { let! data = fetch(); return data }\n```",
            "return" => "**return** - Return value from computation expression",
            "return!" => "**return!** - Return wrapped value from computation expression",
            "yield" => "**yield** - Yield value in sequence expression",
            "yield!" => "**yield!** - Yield sequence of values",
            "do" => "**do** - Execute expression for side effects",
            "do!" => "**do!** - Execute async expression for side effects",
            "while" => "**while** - While loop\n\n```fusabi\nwhile condition do\n  body\n```",
            "break" => "**break** - Exit loop early",
            "continue" => "**continue** - Skip to next iteration",
            "true" => "**true** - Boolean true literal",
            "false" => "**false** - Boolean false literal",
            _ => return None,
        };
        Some(docs.to_string())
    }

    fn get_completions(&self) -> Vec<CompletionItem> {
        let keywords = vec![
            ("let", "Bind a value", CompletionItemKind::KEYWORD),
            ("let rec", "Recursive binding", CompletionItemKind::KEYWORD),
            ("in", "Let body", CompletionItemKind::KEYWORD),
            ("if", "Conditional", CompletionItemKind::KEYWORD),
            ("then", "If true branch", CompletionItemKind::KEYWORD),
            ("else", "If false branch", CompletionItemKind::KEYWORD),
            ("fun", "Lambda function", CompletionItemKind::KEYWORD),
            ("match", "Pattern matching", CompletionItemKind::KEYWORD),
            ("with", "Match arms / record update", CompletionItemKind::KEYWORD),
            ("type", "Type definition", CompletionItemKind::KEYWORD),
            ("module", "Module definition", CompletionItemKind::KEYWORD),
            ("open", "Import module", CompletionItemKind::KEYWORD),
            ("async", "Async block", CompletionItemKind::KEYWORD),
            ("return", "Return value", CompletionItemKind::KEYWORD),
            ("yield", "Yield value", CompletionItemKind::KEYWORD),
            ("do", "Side effect", CompletionItemKind::KEYWORD),
            ("while", "While loop", CompletionItemKind::KEYWORD),
            ("break", "Exit loop", CompletionItemKind::KEYWORD),
            ("continue", "Next iteration", CompletionItemKind::KEYWORD),
            ("true", "Boolean true", CompletionItemKind::CONSTANT),
            ("false", "Boolean false", CompletionItemKind::CONSTANT),
        ];

        let builtins = vec![
            ("printfn", "Print formatted line", CompletionItemKind::FUNCTION),
            ("printf", "Print formatted", CompletionItemKind::FUNCTION),
            ("List.map", "Map over list", CompletionItemKind::FUNCTION),
            ("List.filter", "Filter list", CompletionItemKind::FUNCTION),
            ("List.fold", "Fold list", CompletionItemKind::FUNCTION),
            ("List.head", "First element", CompletionItemKind::FUNCTION),
            ("List.tail", "Rest of list", CompletionItemKind::FUNCTION),
            ("List.length", "List length", CompletionItemKind::FUNCTION),
            ("Array.length", "Array length", CompletionItemKind::FUNCTION),
            ("Array.get", "Get array element", CompletionItemKind::FUNCTION),
            ("Array.set", "Set array element", CompletionItemKind::FUNCTION),
        ];

        keywords
            .into_iter()
            .chain(builtins)
            .map(|(label, detail, kind)| CompletionItem {
                label: label.to_string(),
                kind: Some(kind),
                detail: Some(detail.to_string()),
                ..Default::default()
            })
            .collect()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for FusabiLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        ..Default::default()
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "fusabi-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Fusabi LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        
        {
            let mut docs = self.documents.write().unwrap();
            docs.insert(uri.clone(), text.clone());
        }
        
        self.publish_diagnostics(uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text;
            
            {
                let mut docs = self.documents.write().unwrap();
                docs.insert(uri.clone(), text.clone());
            }
            
            self.publish_diagnostics(uri, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut docs = self.documents.write().unwrap();
        docs.remove(&params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let docs = self.documents.read().unwrap();
        let text = match docs.get(uri) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };
        drop(docs);

        let info = self.get_hover_info(&text, position);
        
        Ok(info.map(|content| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        }))
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let items = self.get_completions();
        Ok(Some(CompletionResponse::Array(items)))
    }
}
