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
    /// Vrai si le symbole est exposé via `export`.
    pub is_exported: bool,
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

    pub fn rebuild(&mut self, source: &str, statements: &[Statement]) {
        self.symbols.clear();

        for statement in statements {
            self.collect_statement(source, statement);
        }
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.values()
    }

    fn insert(&mut self, name: String, kind: SymbolKind, span: Span, is_exported: bool) {
        self.symbols.insert(
            name.clone(),
            Symbol {
                name,
                kind,
                span,
                is_exported,
            },
        );
    }

    fn collect_statement(&mut self, source: &str, statement: &Statement) {
        match statement {
            Statement::Positioned {
                line,
                column,
                statement,
            } => {
                self.collect_positioned(source, statement, *line, *column, false);
            }

            _ => {
                self.collect_positioned(source, statement, 1, 1, false);
            }
        }
    }

    fn collect_positioned(
        &mut self,
        source: &str,
        statement: &Statement,
        line: usize,
        column: usize,
        is_exported: bool,
    ) {
        match statement {
            Statement::Positioned {
                line,
                column,
                statement,
            } => {
                self.collect_positioned(source, statement, *line, *column, is_exported);
            }

            /*
             * `export <stmt>` : on propage `is_exported = true`
             * à l'instruction encapsulée. `Positioned` intermédiaire
             * propage le flag sans le modifier, donc
             * `export const X = 1` et
             * `export { const X = 1 }` marchent tous les deux.
             */
            Statement::Export { statement } => {
                self.collect_positioned(source, statement, line, column, true);
            }

            Statement::Let { name, .. } => {
                let span = find_name_span(source, line, column, name);

                self.insert(name.clone(), SymbolKind::Variable, span, is_exported);
            }

            Statement::Function { name, .. } => {
                let span = find_name_span(source, line, column, name);

                self.insert(name.clone(), SymbolKind::Function, span, is_exported);
            }

            Statement::Class { name, .. } => {
                let span = find_name_span(source, line, column, name);

                self.insert(name.clone(), SymbolKind::Class, span, is_exported);
            }

            Statement::Interface { name, .. } => {
                let span = find_name_span(source, line, column, name);

                self.insert(name.clone(), SymbolKind::Interface, span, is_exported);
            }

            Statement::Import { path } => {
                if let Some(name) = path.last() {
                    let span = find_name_span(source, line, column, name);

                    self.insert(name.clone(), SymbolKind::Import, span, is_exported);
                }
            }

            Statement::FromImport { items, .. } => {
                for item in items {
                    let name = item.alias.as_ref().unwrap_or(&item.name);

                    let span = find_name_span(source, line, column, name);

                    self.insert(name.clone(), SymbolKind::Import, span, is_exported);
                }
            }

            Statement::Block(statements) => {
                for statement in statements {
                    self.collect_statement(source, statement);
                }
            }

            Statement::If {
                then_branch,
                else_branch,
                ..
            } => {
                for statement in then_branch {
                    self.collect_statement(source, statement);
                }

                if let Some(statements) = else_branch {
                    for statement in statements {
                        self.collect_statement(source, statement);
                    }
                }
            }

            Statement::While { body, .. } | Statement::ForIn { body, .. } => {
                for statement in body {
                    self.collect_statement(source, statement);
                }
            }

            Statement::Try {
                try_body,
                catch_body,
                finally_body,
                ..
            } => {
                for statement in try_body {
                    self.collect_statement(source, statement);
                }

                if let Some(statements) = catch_body {
                    for statement in statements {
                        self.collect_statement(source, statement);
                    }
                }

                if let Some(statements) = finally_body {
                    for statement in statements {
                        self.collect_statement(source, statement);
                    }
                }
            }

            _ => {}
        }
    }
}

fn find_name_span(source: &str, line: usize, column: usize, name: &str) -> Span {
    /*
     * `Statement::Positioned` nous donne la position
     * du début de la déclaration, mais pas directement
     * la position du nom.
     *
     * On cherche donc le nom dans la ligne de la
     * déclaration, à partir de la colonne connue.
     */
    let line_index = line.saturating_sub(1);

    let line_text = source.lines().nth(line_index);

    let Some(line_text) = line_text else {
        return span_from_position(source, line, column, name.len());
    };

    /*
     * Les colonnes du parser Kastel sont 1-based.
     * On les convertit en index byte 0-based.
     */
    let search_start = column.saturating_sub(1);

    let search_start = search_start.min(line_text.len());

    /*
     * Cherche d'abord le nom à partir de la
     * colonne de déclaration.
     */
    if let Some(relative) = find_identifier(&line_text[search_start..], name) {
        let name_column = search_start + relative + 1;

        return span_from_position(source, line, name_column, name.len());
    }

    /*
     * Fallback : recherche sur toute la ligne.
     */
    if let Some(relative) = find_identifier(line_text, name) {
        return span_from_position(source, line, relative + 1, name.len());
    }

    /*
     * Dernier fallback pour conserver un span valide
     * même si la source est momentanément incomplète
     * pendant une frappe dans VS Code.
     */
    span_from_position(source, line, column, name.len())
}

fn find_identifier(source: &str, name: &str) -> Option<usize> {
    let mut offset = 0;

    while offset <= source.len() {
        let remaining = &source[offset..];

        let relative = remaining.find(name)?;

        let start = offset + relative;

        let end = start + name.len();

        if is_identifier_boundary(source, start, end) {
            return Some(start);
        }

        offset = end;
    }

    None
}

fn is_identifier_boundary(source: &str, start: usize, end: usize) -> bool {
    let before = source[..start].chars().next_back();

    let after = source[end..].chars().next();

    let before_is_identifier = before.is_some_and(is_identifier_char);

    let after_is_identifier = after.is_some_and(is_identifier_char);

    !before_is_identifier && !after_is_identifier
}

fn is_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    use kastel::frontend::lexer::Lexer;
    use kastel::frontend::parser::Parser;

    fn build_index(source: &str) -> SymbolIndex {
        let mut lexer = Lexer::new(source.to_owned());

        let tokens = lexer.scan_token().expect("lexer failed");

        let mut parser = Parser::new(tokens);

        let statements = parser.parse().expect("parser failed");

        let mut index = SymbolIndex::new();

        index.rebuild(source, &statements);

        index
    }

    #[test]
    fn collects_const_and_function() {
        let source = "const VALUE = 42\n\
             \n\
             func hello() {\n\
                 return VALUE\n\
             }\n";

        let index = build_index(source);

        assert!(index.get("VALUE").is_some(), "VALUE was not indexed");

        assert!(index.get("hello").is_some(), "hello was not indexed");
    }

    #[test]
    fn const_span_points_to_name() {
        let source = "const VALUE = 42\n";

        let index = build_index(source);

        let symbol = index.get("VALUE").expect("VALUE was not indexed");

        assert_eq!(&source[symbol.span.start..symbol.span.end], "VALUE");
    }

    #[test]
    fn function_span_points_to_name() {
        let source = "func hello() {\n\
             }\n";

        let index = build_index(source);

        let symbol = index.get("hello").expect("hello was not indexed");

        assert_eq!(&source[symbol.span.start..symbol.span.end], "hello");
    }

    #[test]
    fn ignores_partial_identifier_match() {
        let source = "const VALUE2 = 10\n\
             const VALUE = 42\n";

        let span = find_name_span(source, 2, 1, "VALUE");

        assert_eq!(&source[span.start..span.end], "VALUE");
    }

    #[test]
    fn private_symbols_are_not_exported() {
        let source = "const PRIVATE = 10\n";

        let index = build_index(source);

        let symbol = index.get("PRIVATE").expect("PRIVATE was not indexed");

        assert!(!symbol.is_exported, "PRIVATE should not be exported");
    }

    #[test]
    fn export_const_marks_symbol_as_exported() {
        let source = "export const VALUE = 42\n";

        let index = build_index(source);

        let symbol = index.get("VALUE").expect("VALUE was not indexed");

        assert!(symbol.is_exported, "VALUE should be exported");
    }

    #[test]
    fn export_function_marks_symbol_as_exported() {
        let source = "export func hello() {\n\
             }\n";

        let index = build_index(source);

        let symbol = index.get("hello").expect("hello was not indexed");

        assert!(symbol.is_exported, "hello should be exported");
    }

    #[test]
    fn mixed_exports_are_flagged_correctly() {
        let source = "export const PUBLIC = 1\n\
             const PRIVATE = 2\n\
             export func shared() {\n\
             }\n";

        let index = build_index(source);

        assert!(index.get("PUBLIC").unwrap().is_exported);

        assert!(!index.get("PRIVATE").unwrap().is_exported);

        assert!(index.get("shared").unwrap().is_exported);
    }
}
