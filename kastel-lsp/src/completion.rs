use serde_json::{Value, json};

use crate::symbols::SymbolKind;
use crate::workspace::Workspace;

pub fn build_completion(
    workspace: &Workspace,
    uri: &str,
    line: u32,
    character: u32,
) -> Option<Value> {
    let document = workspace.get(uri)?;

    let prefix = current_prefix(&document.text, line as usize, character as usize);

    let mut items = Vec::new();

    for keyword in KEYWORDS {
        if keyword.starts_with(prefix) {
            items.push(json!({
                "label": keyword,
                "kind": 14
            }));
        }
    }

    for symbol in document.symbols.iter() {
        if symbol.name.starts_with(prefix) {
            let kind = match symbol.kind {
                SymbolKind::Variable => 6,
                SymbolKind::Function => 3,
                SymbolKind::Class => 7,
                SymbolKind::Interface => 8,
                SymbolKind::Import => 9,
            };

            items.push(json!({
                "label": symbol.name,
                "kind": kind
            }));
        }
    }

    Some(json!({
        "isIncomplete": false,
        "items": items
    }))
}

fn current_prefix(source: &str, line: usize, character: usize) -> &str {
    let Some(line_text) = source.lines().nth(line) else {
        return "";
    };

    let end = character.min(line_text.len());

    let bytes = line_text.as_bytes();

    let mut start = end;

    while start > 0 {
        let byte = bytes[start - 1];

        if byte.is_ascii_alphanumeric() || byte == b'_' {
            start -= 1;
        } else {
            break;
        }
    }

    &line_text[start..end]
}

const KEYWORDS: &[&str] = &[
    "const",
    "let",
    "func",
    "class",
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
