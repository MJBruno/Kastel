use serde_json::{Value, json};

use crate::diagnostics::build_diagnostics;
use crate::document_protocol::{parse_did_change, parse_did_open};
use crate::module_resolver::ModuleResolver;
use crate::protocol::{RpcRequest, RpcResponse};
use crate::uri_util::{path_to_uri, uri_to_path};
use crate::workspace::Workspace;

pub enum ServerMessage {
    Response(RpcResponse),
    Notification(Value),
}

pub struct Server {
    initialized: bool,
    shutdown: bool,
    workspace: Workspace,
    module_resolver: Option<ModuleResolver>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            initialized: false,
            shutdown: false,
            workspace: Workspace::new(),
            module_resolver: None,
        }
    }

    pub fn handle(&mut self, request: RpcRequest) -> Vec<ServerMessage> {
        let mut messages = Vec::new();

        match request.method.as_str() {
            "initialize" => {
                messages.push(ServerMessage::Response(
                    self.initialize(request.id, request.params),
                ));
            }

            "initialized" => {
                self.initialized = true;
            }

            "textDocument/didOpen" => {
                if let Some(notification) = self.did_open(request.params) {
                    messages.push(ServerMessage::Notification(notification));
                }
            }

            "textDocument/didChange" => {
                if let Some(notification) = self.did_change(request.params) {
                    messages.push(ServerMessage::Notification(notification));
                }
            }

            "textDocument/didClose" => {
                if let Some(notification) = self.did_close(request.params) {
                    messages.push(ServerMessage::Notification(notification));
                }
            }

            "textDocument/documentSymbol" => {
                if let Some(message) = self.document_symbols(request.id, request.params) {
                    messages.push(message);
                }
            }

            "textDocument/documentHighlight" => {
                if let Some(message) = self.document_highlight(request.id, request.params) {
                    messages.push(message);
                }
            }

            "textDocument/completion" => {
                if let Some(message) = self.completion(request.id, request.params) {
                    messages.push(message);
                }
            }

            "textDocument/hover" => {
                if let Some(message) = self.hover(request.id, request.params) {
                    messages.push(message);
                }
            }

            "textDocument/definition" => {
                if let Some(message) = self.definition(request.id, request.params) {
                    messages.push(message);
                }
            }

            "textDocument/references" => {
                if let Some(message) = self.references(request.id, request.params) {
                    messages.push(message);
                }
            }

            "textDocument/rename" => {
                if let Some(message) = self.rename(request.id, request.params) {
                    messages.push(message);
                }
            }

            "shutdown" => {
                messages.push(ServerMessage::Response(self.shutdown(request.id)));
            }

            "exit" => {
                self.shutdown = true;
            }

            _ => {
                if let Some(id) = request.id {
                    messages.push(ServerMessage::Response(RpcResponse::new(
                        id,
                        json!({
                            "error": format!(
                                "Unknown LSP method: {}",
                                request.method
                            )
                        }),
                    )));
                }
            }
        }

        messages
    }

    fn initialize(&mut self, id: Option<Value>, params: Option<Value>) -> RpcResponse {
        if let Some(root_uri) = params
            .as_ref()
            .and_then(|value| value.get("rootUri"))
            .and_then(Value::as_str)
        {
            if let Some(path) = uri_to_path(root_uri) {
                self.module_resolver = Some(ModuleResolver::new(Some(path)));
            }
        }

        RpcResponse::new(
            id.unwrap_or(Value::Null),
            json!({
                "capabilities": {
                    "textDocumentSync": 1,
                    "hoverProvider": true,
                    "definitionProvider": true,
                    "referencesProvider": true,
                    "renameProvider": true,
                    "documentSymbolProvider": true,
                    "documentHighlightProvider": true,
                    "completionProvider": {
                        "triggerCharacters": [".", ":"]
                    }
                }
            }),
        )
    }

    /// Construit le message `publishDiagnostics` complet :
    /// lexer + parser (stockés dans le document) + sémantique.
    fn diagnostics_for(&self, uri: &str) -> Option<Value> {
        let document = self.workspace.get(uri)?;

        let mut all = document.diagnostics.clone();

        all.extend(
            crate::semantic::analyze(&self.workspace, uri)
        );

        Some(build_diagnostics(
            uri,
            &document.text,
            all,
        ))
    }

    fn did_open(&mut self, params: Option<Value>) -> Option<Value> {
        let params = params?;

        let (uri, _language_id, version, text) = parse_did_open(&params)?;

        self.workspace.open(uri.clone(), version, text);

        self.load_imports(&uri);

        eprintln!(
            "Opened document: {} ({} document(s))",
            uri,
            self.workspace.len()
        );

        self.diagnostics_for(&uri)
    }

    fn did_change(&mut self, params: Option<Value>) -> Option<Value> {
        let params = params?;

        let (uri, version, text) = parse_did_change(&params)?;

        if !self.workspace.update(&uri, version, text) {
            return None;
        }

        self.load_imports(&uri);

        let document = self.workspace.get(&uri)?;

        eprintln!("Updated document: {} -> version {}", uri, document.version);

        self.diagnostics_for(&uri)
    }

    fn did_close(&mut self, params: Option<Value>) -> Option<Value> {
        let params = params?;

        let uri = params.get("textDocument")?.get("uri")?.as_str()?;

        self.workspace.close(uri);

        eprintln!(
            "Closed document: {} ({} document(s))",
            uri,
            self.workspace.len()
        );

        Some(build_diagnostics(uri, "", Vec::new()))
    }

    fn load_imports(&mut self, uri: &str) {
        let Some(resolver) = self.module_resolver.clone() else {
            return;
        };

        let Some(document) = self.workspace.get(uri) else {
            return;
        };

        let analysis = crate::analyzer::analyze(&document.text);

        for statement in &analysis.statements {
            self.load_statement_imports(uri, statement, &resolver);
        }
    }

    fn load_statement_imports(
        &mut self,
        uri: &str,
        statement: &kastel::frontend::ast::Statement,
        resolver: &ModuleResolver,
    ) {
        use kastel::frontend::ast::Statement;

        match statement {
            Statement::Positioned { statement, .. } => {
                self.load_statement_imports(uri, statement, resolver);
            }

            Statement::Import { path } => {
                let Some(current_file) = uri_to_path(uri) else {
                    return;
                };

                let Some(module_path) = resolver.resolve(&current_file, path) else {
                    eprintln!("Module not found: {}", path.join("."),);

                    return;
                };

                let module_uri = path_to_uri(&module_path);

                if self.workspace.get(&module_uri).is_none() {
                    if let Err(error) = self.workspace.open_file(module_uri.clone(), &module_path) {
                        eprintln!("Failed to load {}: {}", module_uri, error,);
                    }
                }
            }

            _ => {}
        }
    }

    fn document_symbols(&self, id: Option<Value>, params: Option<Value>) -> Option<ServerMessage> {
        let id = id?;
        let params = params?;

        let uri = params
            .get("textDocument")
            .and_then(|value| value.get("uri"))
            .and_then(Value::as_str)?;

        let result = match self.workspace.get(uri) {
            Some(document) => {
                let mut symbols = document.symbols.iter().collect::<Vec<_>>();

                symbols.sort_by_key(|symbol| symbol.span.start);

                symbols
                    .into_iter()
                    .map(|symbol| {
                        let start =
                            crate::lsp_position::offset_to_lsp(&document.text, symbol.span.start);

                        let end =
                            crate::lsp_position::offset_to_lsp(&document.text, symbol.span.end);

                        let kind = match symbol.kind {
                            crate::symbols::SymbolKind::Variable => 13,
                            crate::symbols::SymbolKind::Function => 12,
                            crate::symbols::SymbolKind::Class => 5,
                            crate::symbols::SymbolKind::Interface => 11,
                            crate::symbols::SymbolKind::Import => 2,
                        };

                        json!({
                            "name": symbol.name,
                            "kind": kind,
                            "range": {
                                "start": {
                                    "line": start.0,
                                    "character": start.1
                                },
                                "end": {
                                    "line": end.0,
                                    "character": end.1
                                }
                            },
                            "selectionRange": {
                                "start": {
                                    "line": start.0,
                                    "character": start.1
                                },
                                "end": {
                                    "line": end.0,
                                    "character": end.1
                                }
                            }
                        })
                    })
                    .collect::<Vec<_>>()
            }

            None => Vec::new(),
        };

        Some(ServerMessage::Response(RpcResponse::new(
            id,
            Value::Array(result),
        )))
    }

    fn document_highlight(&self, id: Option<Value>, params: Option<Value>) -> Option<ServerMessage> {
        let id = id?;
        let params = params?;

        let uri = params.get("textDocument")?.get("uri")?.as_str()?;

        let position = params.get("position")?;

        let line = position.get("line")?.as_u64()? as u32;

        let character = position.get("character")?.as_u64()? as u32;

        let result = crate::document_highlight::build_document_highlight(
            &self.workspace,
            uri,
            line,
            character,
        );

        Some(ServerMessage::Response(RpcResponse::new(
            id,
            result.unwrap_or(Value::Null),
        )))
    }

    fn completion(&self, id: Option<Value>, params: Option<Value>) -> Option<ServerMessage> {
        let id = id?;
        let params = params?;

        let uri = params.get("textDocument")?.get("uri")?.as_str()?;

        let position = params.get("position")?;

        let line = position.get("line")?.as_u64()? as u32;

        let character = position.get("character")?.as_u64()? as u32;

        let result = crate::completion::build_completion(&self.workspace, uri, line, character);

        Some(ServerMessage::Response(RpcResponse::new(
            id,
            result.unwrap_or(Value::Null),
        )))
    }

    fn hover(&self, id: Option<Value>, params: Option<Value>) -> Option<ServerMessage> {
        let id = id?;
        let params = params?;

        let uri = params.get("textDocument")?.get("uri")?.as_str()?;

        let position = params.get("position")?;

        let line = position.get("line")?.as_u64()? as u32;

        let character = position.get("character")?.as_u64()? as u32;

        let result = crate::hover::build_hover(&self.workspace, uri, line, character);

        Some(ServerMessage::Response(RpcResponse::new(
            id,
            result.unwrap_or(Value::Null),
        )))
    }

    fn definition(&self, id: Option<Value>, params: Option<Value>) -> Option<ServerMessage> {
        let id = id?;
        let params = params?;

        let uri = params.get("textDocument")?.get("uri")?.as_str()?;

        let position = params.get("position")?;

        let line = position.get("line")?.as_u64()? as u32;

        let character = position.get("character")?.as_u64()? as u32;

        let result = crate::definition::build_definition(&self.workspace, uri, line, character);

        Some(ServerMessage::Response(RpcResponse::new(
            id,
            result.unwrap_or(Value::Null),
        )))
    }

    fn references(&self, id: Option<Value>, params: Option<Value>) -> Option<ServerMessage> {
        let id = id?;
        let params = params?;

        let uri = params.get("textDocument")?.get("uri")?.as_str()?;

        let position = params.get("position")?;

        let line = position.get("line")?.as_u64()? as u32;

        let character = position.get("character")?.as_u64()? as u32;

        let include_declaration = params
            .get("context")
            .and_then(|context| context.get("includeDeclaration"))
            .and_then(Value::as_bool)
            .unwrap_or(true);

        let result = crate::references::build_references(
            &self.workspace,
            uri,
            line,
            character,
            include_declaration,
        )
        .unwrap_or_else(|| Value::Array(Vec::new()));

        Some(ServerMessage::Response(RpcResponse::new(id, result)))
    }

    fn rename(&self, id: Option<Value>, params: Option<Value>) -> Option<ServerMessage> {
        let id = id?;
        let params = params?;

        let uri = params.get("textDocument")?.get("uri")?.as_str()?;

        let position = params.get("position")?;

        let line = position.get("line")?.as_u64()? as u32;

        let character = position.get("character")?.as_u64()? as u32;

        let new_name = params.get("newName")?.as_str()?;

        let result = crate::rename::build_rename(&self.workspace, uri, line, character, new_name);

        Some(ServerMessage::Response(RpcResponse::new(
            id,
            result.unwrap_or(Value::Null),
        )))
    }

    fn shutdown(&mut self, id: Option<Value>) -> RpcResponse {
        self.shutdown = true;

        RpcResponse::new(id.unwrap_or(Value::Null), Value::Null)
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }
}