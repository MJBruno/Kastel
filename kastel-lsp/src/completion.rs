use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::module_resolver::ModuleResolver;
use crate::symbols::SymbolKind;
use crate::workspace::Workspace;

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
     * 1. Mots-clés Kastel.
     */
    add_keyword_completions(
        prefix,
        &mut items,
        &mut seen,
    );

    /*
     * 2. Symboles locaux.
     */
    add_document_completions(
        &document.symbols,
        prefix,
        &mut items,
        &mut seen,
    );

    /*
     * 3. Symboles provenant réellement
     *    des imports du document.
     *
     *    Exemple :
     *
     *    import math.xx
     *
     *    const value = VA
     *
     *    -> VALUE peut être proposé si
     *       xx.ks exporte VALUE.
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
    items: &mut Vec<Value>,
    seen: &mut HashSet<String>,
) {
    for symbol in symbols.iter() {
        if !symbol.name.starts_with(prefix) {
            continue;
        }

        if !seen.insert(
            symbol.name.clone(),
        ) {
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

        let Some(module_uri) =
            resolver.path_to_uri(
                &module_path,
            )
        else {
            continue;
        };

        let Some(module_document) =
            workspace.get(&module_uri)
        else {
            continue;
        };

        /*
         * Uniquement les symboles exportés devraient
         * idéalement être proposés ici.
         *
         * Le SymbolIndex actuel ne contient pas encore
         * la notion d'export. Pour rester compatible
         * avec l'implémentation actuelle, on utilise les
         * symboles du module chargé.
         */
        add_document_completions(
            &module_document.symbols,
            prefix,
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

fn uri_to_path(
    uri: &str,
) -> Option<PathBuf> {
    let path =
        uri.strip_prefix("file:///")?;

    #[cfg(windows)]
    {
        let mut path =
            path.replace('/', "\\");

        if path.len() >= 2
            && path.as_bytes()[0]
                .is_ascii_alphabetic()
            && path.as_bytes()[1] == b':'
        {
            return Some(
                PathBuf::from(path),
            );
        }

        if path.starts_with("\\")
            && !path.starts_with("\\\\")
        {
            path =
                format!("\\{}", path);
        }

        Some(PathBuf::from(path))
    }

    #[cfg(not(windows))]
    {
        Some(
            PathBuf::from(
                format!("/{}", path),
            ),
        )
    }
}

fn current_prefix(
    line: &str,
    byte_index: usize,
) -> &str {
    let end =
        byte_index.min(line.len());

    let bytes =
        line.as_bytes();

    let mut start =
        end;

    while start > 0 {
        let byte =
            bytes[start - 1];

        if byte.is_ascii_alphanumeric()
            || byte == b'_'
        {
            start -= 1;
        } else {
            break;
        }
    }

    &line[start..end]
}

fn utf16_character_to_byte_index(
    text: &str,
    character: usize,
) -> usize {
    if character == 0 {
        return 0;
    }

    let mut utf16_units =
        0usize;

    for (byte_index, ch) in
        text.char_indices()
    {
        let width =
            ch.len_utf16();

        if utf16_units + width
            > character
        {
            return byte_index;
        }

        utf16_units += width;

        if utf16_units == character {
            return byte_index
                + ch.len_utf8();
        }
    }

    text.len()
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

const KEYWORDS: &[&str] = &[
    "const",
    "let",
    "func",
    "class",
    "this",
    "interface",
    "if",
    "else",
    "while",
    "for",
    "in",
    "return",
    "break",
    "continue",
    "import",
    "from",
    "export",
    "try",
    "catch",
    "finally",
    "true",
    "false",
    "None",
];

#[cfg(test)]
mod tests {
    use super::*;

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
    fn empty_prefix_is_supported() {
        let source =
            "const VALUE = 42";

        assert_eq!(
            current_prefix(
                source,
                0
            ),
            ""
        );
    }

    #[test]
    fn prefix_is_extracted() {
        assert_eq!(
            current_prefix(
                "print(VA",
                8
            ),
            "VA"
        );
    }

    #[test]
    fn utf16_position_is_converted() {
        let source =
            "😀abc";

        assert_eq!(
            utf16_character_to_byte_index(
                source,
                2
            ),
            4
        );

        assert_eq!(
            utf16_character_to_byte_index(
                source,
                3
            ),
            5
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
}

