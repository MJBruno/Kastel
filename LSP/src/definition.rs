use serde_json::{Value, json};

use crate::lsp_position::offset_to_lsp;
use crate::text_util::find_word_at;
use crate::workspace::{Workspace, WorkspaceDocument};

fn build_location(uri: &str, document: &WorkspaceDocument, name: &str) -> Option<Value> {
    let symbol = document.symbols.get(name)?;

    let (start_line, start_character) = offset_to_lsp(&document.text, symbol.span.start);

    let (end_line, end_character) = offset_to_lsp(&document.text, symbol.span.end);

    Some(json!({
        "uri": uri,
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

pub fn build_definition(
    workspace: &Workspace,
    uri: &str,
    line: u32,
    character: u32,
) -> Option<Value> {
    let document = workspace.get(uri)?;

    let name = find_word_at(&document.text, line as usize, character as usize)?;

    // 1. Recherche locale.
    if document.symbols.get(name).is_some() {
        return build_location(uri, document, name);
    }

    // 2. Recherche dans les autres documents
    //    déjà chargés dans le workspace.
    for (other_uri, other_document) in workspace.iter() {
        if other_uri == uri {
            continue;
        }

        if other_document.symbols.get(name).is_some() {
            eprintln!("Definition resolved: {} -> {}", name, other_uri);

            return build_location(other_uri, other_document, name);
        }
    }

    None
}
