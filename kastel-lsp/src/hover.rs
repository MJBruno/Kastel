use serde_json::{json, Value};

use crate::lsp_position::offset_to_lsp;
use crate::symbols::SymbolKind;
use crate::workspace::{
    Workspace,
    WorkspaceDocument,
};

pub fn find_word(
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

fn build_hover_value(
    document: &WorkspaceDocument,
    symbol_name: &str,
) -> Option<Value> {
    let symbol =
        document.symbols.get(
            symbol_name,
        )?;

    let kind = match symbol.kind {
        SymbolKind::Variable => {
            "Variable"
        }
        SymbolKind::Function => {
            "Function"
        }
        SymbolKind::Class => {
            "Class"
        }
        SymbolKind::Interface => {
            "Interface"
        }
        SymbolKind::Import => {
            "Import"
        }
    };

    let contents = format!(
        "**{}**\n\n{}",
        symbol.name,
        kind
    );

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

pub fn build_hover(
    workspace: &Workspace,
    uri: &str,
    line: u32,
    character: u32,
) -> Option<Value> {
    let document =
        workspace.get(uri)?;

    let symbol_name =
        find_word(
            &document.text,
            line as usize,
            character as usize,
        )?;

    if document
        .symbols
        .get(symbol_name)
        .is_some()
    {
        return build_hover_value(
            document,
            symbol_name,
        );
    }

    for (
        other_uri,
        other_document,
    ) in workspace.iter()
    {
        if other_uri == uri {
            continue;
        }

        if other_document
            .symbols
            .get(symbol_name)
            .is_some()
        {
            return build_hover_value(
                other_document,
                symbol_name,
            );
        }
    }

    None
}