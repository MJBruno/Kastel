use serde_json::{json, Value};

use crate::lsp_position::offset_to_lsp;
use crate::workspace::Workspace;

pub fn build_references(
    workspace: &Workspace,
    uri: &str,
    line: u32,
    character: u32,
    include_declaration: bool,
) -> Option<Value> {
    let document =
        workspace.get(uri)?;

    let name = find_word(
        &document.text,
        line as usize,
        character as usize,
    )?;

    /*
     * Le symbole doit exister dans le workspace.
     *
     * Il peut être déclaré :
     * - dans le fichier courant ;
     * - dans un module importé déjà chargé.
     */
    let symbol_exists = workspace
        .iter()
        .any(|(_, document)| {
            document
                .symbols
                .get(name)
                .is_some()
        });

    if !symbol_exists {
        return None;
    }

    let mut locations =
        Vec::new();

    /*
     * Parcourt tous les documents actuellement
     * chargés dans le workspace.
     */
    for (
        document_uri,
        document,
    ) in workspace.iter()
    {
        for occurrence in
            find_identifier_occurrences(
                &document.text,
                name,
            )
        {
            let (
                start_line,
                start_character,
            ) = offset_to_lsp(
                &document.text,
                occurrence,
            );

            let end_offset =
                occurrence + name.len();

            let (
                end_line,
                end_character,
            ) = offset_to_lsp(
                &document.text,
                end_offset,
            );

            /*
             * Lorsque la déclaration est exclue,
             * on l'identifie avec le span du symbole
             * du document concerné.
             */
            if !include_declaration {
                if let Some(symbol) =
                    document.symbols.get(name)
                {
                    let (
                        definition_line,
                        definition_character,
                    ) = offset_to_lsp(
                        &document.text,
                        symbol.span.start,
                    );

                    if start_line
                        == definition_line
                        && start_character
                            == definition_character
                    {
                        continue;
                    }
                }
            }

            locations.push(json!({
                "uri": document_uri,
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
            }));
        }
    }

    Some(Value::Array(locations))
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

fn is_identifier_byte(
    byte: u8,
) -> bool {
    byte.is_ascii_alphanumeric()
        || byte == b'_'
}

fn find_identifier_occurrences(
    source: &str,
    name: &str,
) -> Vec<usize> {
    let bytes =
        source.as_bytes();

    let name_bytes =
        name.as_bytes();

    let mut occurrences =
        Vec::new();

    let mut offset =
        0;

    while offset + name_bytes.len()
        <= bytes.len()
    {
        if &bytes[
            offset
                ..offset + name_bytes.len()
        ] == name_bytes
            && is_identifier_boundary(
                source,
                offset,
                offset + name_bytes.len(),
            )
        {
            occurrences.push(offset);

            offset += name_bytes.len();
        } else {
            offset += 1;
        }
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

    let before_is_identifier =
        before.is_some_and(
            is_identifier_char,
        );

    let after_is_identifier =
        after.is_some_and(
            is_identifier_char,
        );

    !before_is_identifier
        && !after_is_identifier
}

fn is_identifier_char(
    c: char,
) -> bool {
    c.is_ascii_alphanumeric()
        || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_all_identifier_occurrences() {
        let source =
            "const VALUE = 42\n\
             print(VALUE)\n\
             VALUE = 43\n";

        let occurrences =
            find_identifier_occurrences(
                source,
                "VALUE",
            );

        assert_eq!(
            occurrences.len(),
            3
        );
    }

    #[test]
    fn ignores_partial_identifier_matches() {
        let source =
            "VALUE\n\
             VALUE2\n\
             MY_VALUE\n\
             VALUE\n";

        let occurrences =
            find_identifier_occurrences(
                source,
                "VALUE",
            );

        assert_eq!(
            occurrences.len(),
            2
        );
    }

    #[test]
    fn find_word_returns_identifier_at_position() {
        let source =
            "print(VALUE)\n";

        let position =
            source
                .find("VALUE")
                .unwrap();

        let result =
            find_word(
                source,
                0,
                position,
            );

        assert_eq!(
            result,
            Some("VALUE")
        );
    }
}