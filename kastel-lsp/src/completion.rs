use std::collections::HashSet;

use serde_json::{json, Value};

use crate::language::KEYWORDS;
use crate::module_resolver::ModuleResolver;
use crate::symbols::{Symbol, SymbolIndex, SymbolKind};
use crate::text_util::{
    current_prefix,
    utf16_character_to_byte_index,
};
use crate::uri_util::{
    path_to_uri,
    uri_to_path,
};
use crate::workspace::Workspace;

/// Portée d'un symbole dans la liste de complétion.
///
/// Sert à :
/// - filtrer : seuls les symboles exportés sont proposés depuis un import ;
/// - trier (`sortText`) : local < importé < mot-clé ;
/// - décrire (`detail`) : « Function » vs « Imported Function ».
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    Local,
    Imported,
}

pub fn build_completion(
    workspace: &Workspace,
    uri: &str,
    line: u32,
    character: u32,
) -> Option<Value> {
    let document = workspace.get(uri)?;

    let line_text =
        document
            .text
            .lines()
            .nth(line as usize)
            .unwrap_or("");

    let byte_index =
        utf16_character_to_byte_index(
            line_text,
            character as usize,
        );

    let prefix =
        current_prefix(
            line_text,
            byte_index,
        );

    let mut items = Vec::new();
    let mut seen =
        HashSet::<String>::new();

    /*
     * 1. Mots-clés Kastel (avec snippets).
     */
    add_keyword_completions(
        prefix,
        &mut items,
        &mut seen,
    );

    /*
     * 2. Symboles du document courant.
     *
     *    Scope::Local : tous, y compris les symboles privés.
     */
    add_document_completions(
        &document.symbols,
        prefix,
        Scope::Local,
        &mut items,
        &mut seen,
    );

    /*
     * 3. Symboles provenant des imports du document.
     *
     *    Scope::Imported : uniquement les symboles exportés.
     */
    add_import_completions(
        workspace,
        uri,
        prefix,
        &mut items,
        &mut seen,
    );

    Some(json!({
        "isIncomplete": false,
        "items": items
    }))
}

fn add_keyword_completions(
    prefix: &str,
    items: &mut Vec<Value>,
    seen: &mut HashSet<String>,
) {
    for keyword in KEYWORDS {
        if !keyword.starts_with(prefix) {
            continue;
        }

        if !seen.insert(
            (*keyword).to_string(),
        ) {
            continue;
        }

        items.push(keyword_completion(keyword));
    }
}

fn keyword_completion(keyword: &str) -> Value {
    match keyword_snippet(keyword) {
        Some(snippet) => json!({
            "label": keyword,
            "kind": 15,               // Snippet
            "detail": "Kastel snippet",
            "insertText": snippet,
            "insertTextFormat": 2,    // Snippet
            "sortText": format!("2_{}", keyword)
        }),
        None => json!({
            "label": keyword,
            "kind": 14,               // Keyword
            "detail": "Kastel keyword",
            "insertText": keyword,
            "sortText": format!("2_{}", keyword)
        }),
    }
}

/// Snippet associé à un mot-clé structurant.
/// `None` pour les mots-clés qui ne doivent pas être expansés
/// (`true`, `false`, `None`, `in`, `break`, `continue`...).
fn keyword_snippet(
    keyword: &str,
) -> Option<&'static str> {
    Some(match keyword {
        "const" => "const ${1:NAME} = ${0}",
        "let" => "let ${1:name} = ${0}",
        "func" => "func ${1:name}(${2}) {\n\t${0}\n}",
        "class" => "class ${1:Name} {\n\t${0}\n}",
        "interface" => "interface ${1:Name} {\n\t${0}\n}",
        "if" => "if ${1:condition} {\n\t${0}\n}",
        "else" => "else {\n\t${0}\n}",
        "while" => "while ${1:condition} {\n\t${0}\n}",
        "for" => "for ${1:item} in ${2:collection} {\n\t${0}\n}",
        "import" => "import ${1:module.path}",
        "export" => "export ${0}",
        "try" => "try {\n\t${0}\n} catch ${1:err} {\n\t\n}",
        "return" => "return ${0}",
        _ => return None,
    })
}

fn add_document_completions(
    symbols: &SymbolIndex,
    prefix: &str,
    scope: Scope,
    items: &mut Vec<Value>,
    seen: &mut HashSet<String>,
) {
    for symbol in symbols.iter() {
        /*
         * Depuis un import : seuls les symboles exportés
         * sont visibles.
         */
        if scope == Scope::Imported
            && !symbol.is_exported
        {
            continue;
        }

        if !symbol.name.starts_with(prefix) {
            continue;
        }

        if !seen.insert(
            symbol.name.clone(),
        ) {
            continue;
        }

        items.push(symbol_completion(symbol, scope));
    }
}

fn symbol_completion(
    symbol: &Symbol,
    scope: Scope,
) -> Value {
    let (sort_prefix, detail_prefix) =
        match scope {
            Scope::Local => ("0_", ""),
            Scope::Imported => ("1_", "Imported "),
        };

    json!({
        "label": symbol.name,
        "kind": completion_kind(symbol.kind),
        "detail": format!(
            "{}{}",
            detail_prefix,
            completion_detail(symbol.kind)
        ),
        "insertText": symbol.name,
        "sortText": format!(
            "{}{}",
            sort_prefix,
            symbol.name
        )
    })
}

fn add_import_completions(
    workspace: &Workspace,
    uri: &str,
    prefix: &str,
    items: &mut Vec<Value>,
    seen: &mut HashSet<String>,
) {
    let Some(document) =
        workspace.get(uri)
    else {
        return;
    };

    let current_file =
        uri_to_path(uri);

    let resolver =
        ModuleResolver::new(None);

    let imports =
        parse_imports(&document.text);

    for import in imports {
        let Some(current_file) =
            current_file.as_deref()
        else {
            continue;
        };

        let Some(module_path) =
            resolver.resolve(
                current_file,
                &import.parts,
            )
        else {
            continue;
        };

        let module_uri =
            path_to_uri(&module_path);

        let Some(module_document) =
            workspace.get(&module_uri)
        else {
            continue;
        };

        add_document_completions(
            &module_document.symbols,
            prefix,
            Scope::Imported,
            items,
            seen,
        );
    }
}

#[derive(Debug)]
struct ImportPath {
    parts: Vec<String>,
}

fn parse_imports(
    source: &str,
) -> Vec<ImportPath> {
    let mut imports = Vec::new();

    for raw_line in source.lines() {
        let line =
            raw_line.trim();

        let Some(rest) =
            line.strip_prefix("import ")
        else {
            continue;
        };

        let path =
            rest.trim()
                .trim_end_matches(';');

        if path.is_empty() {
            continue;
        }

        let parts =
            path.split('.')
                .map(str::trim)
                .filter(|part| {
                    !part.is_empty()
                })
                .map(str::to_string)
                .collect::<Vec<_>>();

        if parts.is_empty() {
            continue;
        }

        imports.push(
            ImportPath {
                parts,
            },
        );
    }

    imports
}

fn completion_kind(
    kind: SymbolKind,
) -> u32 {
    match kind {
        SymbolKind::Variable => 6,
        SymbolKind::Function => 3,
        SymbolKind::Class => 7,
        SymbolKind::Interface => 8,
        SymbolKind::Import => 9,
    }
}

fn completion_detail(
    kind: SymbolKind,
) -> &'static str {
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

    fn build_index(
        source: &str,
    ) -> SymbolIndex {
        let mut lexer =
            Lexer::new(
                source.to_owned(),
            );

        let tokens =
            lexer
                .scan_token()
                .expect(
                    "lexer failed",
                );

        let mut parser =
            Parser::new(tokens);

        let statements =
            parser
                .parse()
                .expect(
                    "parser failed",
                );

        let mut index =
            SymbolIndex::new();

        index.rebuild(
            source,
            &statements,
        );

        index
    }

    fn labels(
        items: &[Value],
    ) -> Vec<String> {
        items
            .iter()
            .filter_map(|item| {
                item["label"]
                    .as_str()
                    .map(str::to_string)
            })
            .collect()
    }

    fn sort_text(
        item: &Value,
    ) -> &str {
        item["sortText"]
            .as_str()
            .expect("missing sortText")
    }

    #[test]
    fn parses_simple_import() {
        let imports =
            parse_imports(
                "import math.xx\n",
            );

        assert_eq!(
            imports.len(),
            1
        );

        assert_eq!(
            imports[0].parts,
            vec![
                "math",
                "xx"
            ]
        );
    }

    #[test]
    fn parses_multiple_imports() {
        let imports =
            parse_imports(
                "import math.xx\n\
                 import utils.print\n",
            );

        assert_eq!(
            imports.len(),
            2
        );

        assert_eq!(
            imports[1].parts,
            vec![
                "utils",
                "print"
            ]
        );
    }

    #[test]
    fn completion_kind_mapping_is_stable() {
        assert_eq!(
            completion_kind(
                SymbolKind::Variable
            ),
            6
        );

        assert_eq!(
            completion_kind(
                SymbolKind::Function
            ),
            3
        );

        assert_eq!(
            completion_kind(
                SymbolKind::Class
            ),
            7
        );
    }

    #[test]
    fn local_completions_include_private_symbols() {
        let source =
            "const PRIVATE = 1\n\
             export const PUBLIC = 2\n";

        let symbols =
            build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(
            &symbols,
            "",
            Scope::Local,
            &mut items,
            &mut seen,
        );

        let labels =
            labels(&items);

        assert!(
            labels.contains(
                &"PRIVATE".to_string()
            )
        );

        assert!(
            labels.contains(
                &"PUBLIC".to_string()
            )
        );
    }

    #[test]
    fn imported_completions_exclude_private_symbols() {
        let source =
            "const PRIVATE = 1\n\
             export const PUBLIC = 2\n";

        let symbols =
            build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(
            &symbols,
            "",
            Scope::Imported,
            &mut items,
            &mut seen,
        );

        let labels =
            labels(&items);

        assert!(
            labels.contains(
                &"PUBLIC".to_string()
            ),
            "PUBLIC should be proposed, got {:?}",
            labels
        );

        assert!(
            !labels.contains(
                &"PRIVATE".to_string()
            ),
            "PRIVATE should NOT be proposed, got {:?}",
            labels
        );
    }

    #[test]
    fn imported_completions_respect_prefix() {
        let source =
            "export const VALUE = 1\n\
             export const OTHER = 2\n";

        let symbols =
            build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(
            &symbols,
            "VA",
            Scope::Imported,
            &mut items,
            &mut seen,
        );

        let labels =
            labels(&items);

        assert_eq!(labels, vec!["VALUE".to_string()]);
    }

    #[test]
    fn local_symbols_sort_before_imported() {
        let source =
            "const LOCAL = 1\n";

        let symbols =
            build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(
            &symbols,
            "",
            Scope::Local,
            &mut items,
            &mut seen,
        );

        assert_eq!(
            sort_text(&items[0]),
            "0_LOCAL"
        );
    }

    #[test]
    fn imported_symbols_sort_with_prefix_one() {
        let source =
            "export const REMOTE = 1\n";

        let symbols =
            build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(
            &symbols,
            "",
            Scope::Imported,
            &mut items,
            &mut seen,
        );

        assert_eq!(
            sort_text(&items[0]),
            "1_REMOTE"
        );
    }

    #[test]
    fn imported_symbol_detail_is_prefixed() {
        let source =
            "export func greet() {\n\
             }\n";

        let symbols =
            build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(
            &symbols,
            "",
            Scope::Imported,
            &mut items,
            &mut seen,
        );

        assert_eq!(
            items[0]["detail"],
            "Imported Function"
        );
    }

    #[test]
    fn local_symbol_detail_has_no_prefix() {
        let source =
            "func greet() {\n\
             }\n";

        let symbols =
            build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(
            &symbols,
            "",
            Scope::Local,
            &mut items,
            &mut seen,
        );

        assert_eq!(
            items[0]["detail"],
            "Function"
        );
    }

    #[test]
    fn keyword_func_is_a_snippet() {
        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_keyword_completions(
            "func",
            &mut items,
            &mut seen,
        );

        assert_eq!(items.len(), 1);

        let item = &items[0];

        assert_eq!(item["label"], "func");
        assert_eq!(item["kind"], 15);
        assert_eq!(item["insertTextFormat"], 2);
        assert_eq!(
            item["insertText"],
            "func ${1:name}(${2}) {\n\t${0}\n}"
        );
        assert_eq!(item["sortText"], "2_func");
    }

    #[test]
    fn keyword_true_is_not_a_snippet() {
        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_keyword_completions(
            "true",
            &mut items,
            &mut seen,
        );

        assert_eq!(items.len(), 1);

        let item = &items[0];

        assert_eq!(item["label"], "true");
        assert_eq!(item["kind"], 14);
        // Pas de `insertTextFormat` pour un keyword simple.
        assert!(item.get("insertTextFormat").is_none());
    }

    #[test]
    fn keyword_completion_respects_prefix() {
        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_keyword_completions(
            "wh",
            &mut items,
            &mut seen,
        );

        let labels =
            labels(&items);

        assert_eq!(
            labels,
            vec!["while".to_string()]
        );
    }
}