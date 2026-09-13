use std::collections::HashMap;

use kastel::frontend::ast::Statement;

use crate::source_position::span_from_position;
use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,
    Function,
    Class,
    Interface,
    Import,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolIndex {
    symbols: HashMap<String, Symbol>,
}

impl SymbolIndex {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn rebuild(
        &mut self,
        source: &str,
        statements: &[Statement],
    ) {
        self.symbols.clear();

        for statement in statements {
            self.collect_statement(
                source,
                statement,
            );
        }
    }

    pub fn get(
        &self,
        name: &str,
    ) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = &Symbol> {
        self.symbols.values()
    }

    fn insert(
        &mut self,
        name: String,
        kind: SymbolKind,
        span: Span,
    ) {
        self.symbols.insert(
            name.clone(),
            Symbol {
                name,
                kind,
                span,
            },
        );
    }

    fn collect_statement(
        &mut self,
        source: &str,
        statement: &Statement,
    ) {
        match statement {
            Statement::Positioned {
                line,
                column,
                statement,
            } => {
                self.collect_positioned(
                    source,
                    statement,
                    *line,
                    *column,
                );
            }

            _ => {
                self.collect_positioned(
                    source,
                    statement,
                    1,
                    1,
                );
            }
        }
    }

    fn collect_positioned(
        &mut self,
        source: &str,
        statement: &Statement,
        line: usize,
        column: usize,
    ) {
        match statement {
            Statement::Positioned {
                line,
                column,
                statement,
            } => {
                self.collect_positioned(
                    source,
                    statement,
                    *line,
                    *column,
                );
            }

            Statement::Let {
                name,
                ..
            } => {
                let span =
                    span_from_position(
                        source,
                        line,
                        column,
                        name.len(),
                    );

                self.insert(
                    name.clone(),
                    SymbolKind::Variable,
                    span,
                );
            }

            Statement::Function {
                name,
                ..
            } => {
                let span =
                    span_from_position(
                        source,
                        line,
                        column,
                        name.len(),
                    );

                self.insert(
                    name.clone(),
                    SymbolKind::Function,
                    span,
                );
            }

            Statement::Class {
                name,
                ..
            } => {
                let span =
                    span_from_position(
                        source,
                        line,
                        column,
                        name.len(),
                    );

                self.insert(
                    name.clone(),
                    SymbolKind::Class,
                    span,
                );
            }

            Statement::Interface {
                name,
                ..
            } => {
                let span =
                    span_from_position(
                        source,
                        line,
                        column,
                        name.len(),
                    );

                self.insert(
                    name.clone(),
                    SymbolKind::Interface,
                    span,
                );
            }

            Statement::Import {
                path,
            } => {
                if let Some(name) =
                    path.last()
                {
                    let span =
                        span_from_position(
                            source,
                            line,
                            column,
                            name.len(),
                        );

                    self.insert(
                        name.clone(),
                        SymbolKind::Import,
                        span,
                    );
                }
            }

            Statement::FromImport {
                items,
                ..
            } => {
                for item in items {
                    let name =
                        item.alias
                            .as_ref()
                            .unwrap_or(
                                &item.name,
                            );

                    let span =
                        span_from_position(
                            source,
                            line,
                            column,
                            name.len(),
                        );

                    self.insert(
                        name.clone(),
                        SymbolKind::Import,
                        span,
                    );
                }
            }

            Statement::Export {
                statement,
            } => {
                self.collect_positioned(
                    source,
                    statement,
                    line,
                    column,
                );
            }

            Statement::Block(
                statements,
            ) => {
                for statement in statements {
                    self.collect_statement(
                        source,
                        statement,
                    );
                }
            }

            Statement::If {
                then_branch,
                else_branch,
                ..
            } => {
                for statement in then_branch {
                    self.collect_statement(
                        source,
                        statement,
                    );
                }

                if let Some(statements) =
                    else_branch
                {
                    for statement in statements {
                        self.collect_statement(
                            source,
                            statement,
                        );
                    }
                }
            }

            Statement::While {
                body,
                ..
            }
            | Statement::ForIn {
                body,
                ..
            } => {
                for statement in body {
                    self.collect_statement(
                        source,
                        statement,
                    );
                }
            }

            Statement::Try {
                try_body,
                catch_body,
                finally_body,
                ..
            } => {
                for statement in try_body {
                    self.collect_statement(
                        source,
                        statement,
                    );
                }

                if let Some(statements) =
                    catch_body
                {
                    for statement in statements {
                        self.collect_statement(
                            source,
                            statement,
                        );
                    }
                }

                if let Some(statements) =
                    finally_body
                {
                    for statement in statements {
                        self.collect_statement(
                            source,
                            statement,
                        );
                    }
                }
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_const_and_function() {
        let source =
            "const VALUE = 42\n\
             \n\
             func hello() {\n\
                 return VALUE\n\
             }\n";

        let mut lexer =
            kastel::frontend::lexer::Lexer::new(
                source.to_owned(),
            );

        let tokens =
            lexer
                .scan_token()
                .expect("lexer failed");

        let mut parser =
            kastel::frontend::parser::Parser::new(
                tokens,
            );

        let statements =
            parser
                .parse()
                .expect("parser failed");

        let mut index =
            SymbolIndex::new();

        index.rebuild(
            source,
            &statements,
        );

        assert!(
            index.get("VALUE").is_some(),
            "VALUE was not indexed"
        );

        assert!(
            index.get("hello").is_some(),
            "hello was not indexed"
        );
    }
}