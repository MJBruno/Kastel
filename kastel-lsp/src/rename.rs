use serde_json::{json, Value};

use crate::lsp_position::offset_to_lsp;
use crate::workspace::Workspace;

pub fn build_rename(
    workspace: &Workspace,
    uri: &str,
    line: u32,
    character: u32,
    new_name: &str,
) -> Option<Value> {
    if !is_valid_identifier(new_name) {
        return None;
    }

    let document =
        workspace.get(uri)?;

    let old_name = find_word(
        &document.text,
        line as usize,
        character as usize,
    )?;

    /*
     * Le symbole peut être local au document courant
     * ou déclaré dans un module déjà chargé.
     */
    let symbol_exists = document
        .symbols
        .get(old_name)
        .is_some()
        || workspace
            .iter()
            .any(|(_, document)| {
                document.symbols.get(old_name).is_some()
            });

    if !symbol_exists {
        return None;
    }

    /*
     * Le renommage retourne un WorkspaceEdit :
     *
     * {
     *   "changes": {
     *       "file:///...": [ edits ],
     *       "file:///...": [ edits ]
     *   }
     * }
     *
     * On parcourt tous les documents actuellement
     * présents dans le workspace.
     */
    let mut changes =
        serde_json::Map::new();

    for (document_uri, document) in workspace.iter() {
        let occurrences =
            find_identifier_occurrences(
                &document.text,
                old_name,
            );

        if occurrences.is_empty() {
            continue;
        }

        let edits =
            occurrences
                .into_iter()
                .map(|offset| {
                    let start =
                        offset_to_lsp(
                            &document.text,
                            offset,
                        );

                    let end =
                        offset_to_lsp(
                            &document.text,
                            offset + old_name.len(),
                        );

                    json!({
                        "range": {
                            "start": {
                                "line": start.0,
                                "character": start.1
                            },
                            "end": {
                                "line": end.0,
                                "character": end.1
                            }
                        },
                        "newText": new_name
                    })
                })
                .collect::<Vec<_>>();

        changes.insert(
            document_uri.clone(),
            Value::Array(edits),
        );
    }

    if changes.is_empty() {
        return None;
    }

    Some(json!({
        "changes": changes
    }))
}

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

fn find_identifier_occurrences(
    source: &str,
    name: &str,
) -> Vec<usize> {
    let mut occurrences =
        Vec::new();

    let mut offset =
        0;

    while offset < source.len() {
        let Some(relative) =
            source[offset..].find(name)
        else {
            break;
        };

        let start =
            offset + relative;

        let end =
            start + name.len();

        if is_identifier_boundary(
            source,
            start,
            end,
        ) {
            occurrences.push(start);
        }

        offset = end;
    }

    occurrences
}

fn is_identifier_boundary(
    source: &str,
    start: usize,
    end: usize,
) -> bool {
    let before =
        source[..start]
            .chars()
            .next_back();

    let after =
        source[end..]
            .chars()
            .next();

    !before.is_some_and(
        is_identifier_char,
    )
        && !after.is_some_and(
            is_identifier_char,
        )
}

fn is_identifier_byte(
    byte: u8,
) -> bool {
    byte.is_ascii_alphanumeric()
        || byte == b'_'
}

fn is_identifier_char(
    c: char,
) -> bool {
    c.is_ascii_alphanumeric()
        || c == '_'
}

fn is_valid_identifier(
    name: &str,
) -> bool {
    let mut chars =
        name.chars();

    let Some(first) =
        chars.next()
    else {
        return false;
    };

    if !(first.is_ascii_alphabetic()
        || first == '_')
    {
        return false;
    }

    chars.all(|c| {
        c.is_ascii_alphanumeric()
            || c == '_'
    })
}