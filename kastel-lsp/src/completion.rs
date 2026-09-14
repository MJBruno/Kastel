use std::collections::HashSet;

use serde_json::{Value, json};

use crate::language::KEYWORDS;
use crate::module_resolver::ModuleResolver;
use crate::symbols::SymbolKind;
use crate::text_util::{current_prefix, utf16_character_to_byte_index};
use crate::uri_util::{path_to_uri, uri_to_path};
use crate::workspace::Workspace;

pub fn build_completion(
    workspace: &Workspace,
    uri: &str,
    line: u32,
    character: u32,
) -> Option<Value> {
    let document = workspace.get(uri)?;

    let line_text = document.text.lines().nth(line as usize).unwrap_or("");

    let byte_index = utf16_character_to_byte_index(line_text, character as usize);

    let prefix = current_prefix(line_text, byte_index);

    let mut items = Vec::new();
    let mut seen = HashSet::<String>::new();

    /*
     * 1. Mots-clés Kastel.
     */
    add_keyword_completions(prefix, &mut items, &mut seen);

    /*
     * 2. Symboles locaux.
     *
     *    `only_exported = false` : dans le document courant,
     *    même les symboles privés sont utilisables.
     */
    add_document_completions(&document.symbols, prefix, false, &mut items, &mut seen);

    /*
     * 3. Symboles provenant réellement des imports du document.
     *
     *    `only_exported = true` : on n'expose que les symboles
     *    déclarés avec `export` dans le module importé.
     */
    add_import_completions(workspace, uri, prefix, &mut items, &mut seen);

    Some(json!({
        "isIncomplete": false,
        "items": items
    }))
}

fn add_keyword_completions(prefix: &str, items: &mut Vec<Value>, seen: &mut HashSet<String>) {
    for keyword in KEYWORDS {
        if !keyword.starts_with(prefix) {
            continue;
        }

        if !seen.insert((*keyword).to_string()) {
            continue;
        }

        items.push(json!({
            "label": keyword,
            "kind": 14,
            "detail": "Kastel keyword",
            "insertText": keyword
        }));
    }
}

fn add_document_completions(
    symbols: &crate::symbols::SymbolIndex,
    prefix: &str,
    only_exported: bool,
    items: &mut Vec<Value>,
    seen: &mut HashSet<String>,
) {
    for symbol in symbols.iter() {
        if only_exported && !symbol.is_exported {
            continue;
        }

        if !symbol.name.starts_with(prefix) {
            continue;
        }

        if !seen.insert(symbol.name.clone()) {
            continue;
        }

        items.push(json!({
            "label": symbol.name,
            "kind": completion_kind(symbol.kind),
            "detail": completion_detail(symbol.kind),
            "insertText": symbol.name
        }));
    }
}

fn add_import_completions(
    workspace: &Workspace,
    uri: &str,
    prefix: &str,
    items: &mut Vec<Value>,
    seen: &mut HashSet<String>,
) {
    let Some(document) = workspace.get(uri) else {
        return;
    };

    let current_file = uri_to_path(uri);

    let resolver = ModuleResolver::new(None);

    let imports = parse_imports(&document.text);

    for import in imports {
        let Some(current_file) = current_file.as_deref() else {
            continue;
        };

        let Some(module_path) = resolver.resolve(current_file, &import.parts) else {
            continue;
        };

        let module_uri = path_to_uri(&module_path);

        let Some(module_document) = workspace.get(&module_uri) else {
            continue;
        };

        /*
         * Seuls les symboles exportés du module importé sont
         * proposés. Les symboles privés (`const X = 1` sans
         * `export`) restent locaux au module.
         */
        add_document_completions(&module_document.symbols, prefix, true, items, seen);
    }
}

#[derive(Debug)]
struct ImportPath {
    parts: Vec<String>,
}

fn parse_imports(source: &str) -> Vec<ImportPath> {
    let mut imports = Vec::new();

    for raw_line in source.lines() {
        let line = raw_line.trim();

        let Some(rest) = line.strip_prefix("import ") else {
            continue;
        };

        let path = rest.trim().trim_end_matches(';');

        if path.is_empty() {
            continue;
        }

        let parts = path
            .split('.')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();

        if parts.is_empty() {
            continue;
        }

        imports.push(ImportPath { parts });
    }

    imports
}

fn completion_kind(kind: SymbolKind) -> u32 {
    match kind {
        SymbolKind::Variable => 6,
        SymbolKind::Function => 3,
        SymbolKind::Class => 7,
        SymbolKind::Interface => 8,
        SymbolKind::Import => 9,
    }
}

fn completion_detail(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Variable => "Variable",
        SymbolKind::Function => "Function",
        SymbolKind::Class => "Class",
        SymbolKind::Interface => "Interface",
        SymbolKind::Import => "Import",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kastel::frontend::lexer::Lexer;
    use kastel::frontend::parser::Parser;

    use crate::symbols::SymbolIndex;

    fn build_index(source: &str) -> SymbolIndex {
        let mut lexer = Lexer::new(source.to_owned());

        let tokens = lexer.scan_token().expect("lexer failed");

        let mut parser = Parser::new(tokens);

        let statements = parser.parse().expect("parser failed");

        let mut index = SymbolIndex::new();

        index.rebuild(source, &statements);

        index
    }

    fn labels(items: &[Value]) -> Vec<String> {
        items
            .iter()
            .filter_map(|item| item["label"].as_str().map(str::to_string))
            .collect()
    }

    #[test]
    fn parses_simple_import() {
        let imports = parse_imports("import math.xx\n");

        assert_eq!(imports.len(), 1);

        assert_eq!(imports[0].parts, vec!["math", "xx"]);
    }

    #[test]
    fn parses_multiple_imports() {
        let imports = parse_imports(
            "import math.xx\n\
                 import utils.print\n",
        );

        assert_eq!(imports.len(), 2);

        assert_eq!(imports[1].parts, vec!["utils", "print"]);
    }

    #[test]
    fn completion_kind_mapping_is_stable() {
        assert_eq!(completion_kind(SymbolKind::Variable), 6);

        assert_eq!(completion_kind(SymbolKind::Function), 3);

        assert_eq!(completion_kind(SymbolKind::Class), 7);
    }

    #[test]
    fn local_completions_include_private_symbols() {
        let source = "const PRIVATE = 1\n\
             export const PUBLIC = 2\n";

        let symbols = build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(&symbols, "", false, &mut items, &mut seen);

        let labels = labels(&items);

        assert!(labels.contains(&"PRIVATE".to_string()));

        assert!(labels.contains(&"PUBLIC".to_string()));
    }

    #[test]
    fn imported_completions_exclude_private_symbols() {
        let source = "const PRIVATE = 1\n\
             export const PUBLIC = 2\n";

        let symbols = build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(&symbols, "", true, &mut items, &mut seen);

        let labels = labels(&items);

        assert!(
            labels.contains(&"PUBLIC".to_string()),
            "PUBLIC should be proposed, got {:?}",
            labels
        );

        assert!(
            !labels.contains(&"PRIVATE".to_string()),
            "PRIVATE should NOT be proposed, got {:?}",
            labels
        );
    }

    #[test]
    fn imported_completions_respect_prefix() {
        let source = "export const VALUE = 1\n\
             export const OTHER = 2\n";

        let symbols = build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(&symbols, "VA", true, &mut items, &mut seen);

        let labels = labels(&items);

        assert_eq!(labels, vec!["VALUE".to_string()]);
    }
}
