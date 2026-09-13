use serde_json::{json, Value};

use crate::lsp_position::offset_to_lsp;
use crate::workspace::{Workspace, WorkspaceDocument};

fn find_word(
    source: &str,
    line: usize,
    character: usize,
) -> Option<&str> {
    let line_text =
        source.lines().nth(line)?;

    if character > line_text.len() {
        return None;
    }

    let bytes =
        line_text.as_bytes();

    let mut start =
        character;

    let mut end =
        character;

    while start > 0
        && is_identifier_byte(
            bytes[start - 1],
        )
    {
        start -= 1;
    }

    while end < bytes.len()
        && is_identifier_byte(
            bytes[end],
        )
    {
        end += 1;
    }

    if start == end {
        return None;
    }

    Some(
        &line_text[start..end]
    )
}

fn is_identifier_byte(
    byte: u8,
) -> bool {
    byte.is_ascii_alphanumeric()
        || byte == b'_'
}

fn build_location(
    uri: &str,
    document: &WorkspaceDocument,
    name: &str,
) -> Option<Value> {
    let symbol =
        document.symbols.get(name)?;

    let (
        start_line,
        start_character,
    ) = offset_to_lsp(
        &document.text,
        symbol.span.start,
    );

    let (
        end_line,
        end_character,
    ) = offset_to_lsp(
        &document.text,
        symbol.span.end,
    );

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
    let document =
        workspace.get(uri)?;

    let name =
        find_word(
            &document.text,
            line as usize,
            character as usize,
        )?;

    // 1. Recherche locale.
    if document.symbols.get(name).is_some() {
        return build_location(
            uri,
            document,
            name,
        );
    }

    // 2. Recherche dans les autres documents
    //    déjà chargés dans le workspace.
    for (other_uri, other_document)
        in workspace.iter()
    {
        if other_uri == uri {
            continue;
        }

        if other_document
            .symbols
            .get(name)
            .is_some()
        {
            eprintln!(
                "Definition resolved: {} -> {}",
                name,
                other_uri
            );

            return build_location(
                other_uri,
                other_document,
                name,
            );
        }
    }

    None
}