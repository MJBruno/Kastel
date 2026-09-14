use serde_json::{Value, json};

use crate::lsp_position::offset_to_lsp;
use crate::symbols::SymbolKind;
use crate::text_util::find_word_at;
use crate::workspace::{Workspace, WorkspaceDocument};

fn build_hover_value(document: &WorkspaceDocument, symbol_name: &str) -> Option<Value> {
    let symbol = document.symbols.get(symbol_name)?;

    let kind = match symbol.kind {
        SymbolKind::Variable => "Variable",
        SymbolKind::Function => "Function",
        SymbolKind::Class => "Class",
        SymbolKind::Interface => "Interface",
        SymbolKind::Import => "Import",
    };

    let contents = format!("**{}**\n\n{}", symbol.name, kind);

    let (start_line, start_character) = offset_to_lsp(&document.text, symbol.span.start);

    let (end_line, end_character) = offset_to_lsp(&document.text, symbol.span.end);

    Some(json!({
        "contents": {
            "kind": "markdown",
            "value": contents
        },
        "range": {
            "start": {
                "line": start_line,
                "character": start_character
            },
            "end": {
                "line": end_line,
                "character": end_character
            }
        }
    }))
}

pub fn build_hover(workspace: &Workspace, uri: &str, line: u32, character: u32) -> Option<Value> {
    let document = workspace.get(uri)?;

    let symbol_name = find_word_at(&document.text, line as usize, character as usize)?;

    if document.symbols.get(symbol_name).is_some() {
        return build_hover_value(document, symbol_name);
    }

    for (other_uri, other_document) in workspace.iter() {
        if other_uri == uri {
            continue;
        }

        if other_document.symbols.get(symbol_name).is_some() {
            return build_hover_value(other_document, symbol_name);
        }
    }

    None
}
