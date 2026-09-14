use serde_json::{json, Value};

use crate::analyzer::Diagnostic;
use crate::position::kastel_to_lsp;

pub fn build_diagnostics(
    uri: &str,
    source: &str,
    diagnostics: Vec<Diagnostic>,
) -> Value {
    let diagnostics = diagnostics
        .into_iter()
        .map(|diagnostic| {
            let (line, character) =
                kastel_to_lsp(
                    source,
                    diagnostic.line,
                    diagnostic.column,
                );

            let end_character =
                diagnostic_end_character(
                    source,
                    diagnostic.line,
                    diagnostic.column,
                    character,
                );

            json!({
                "range": {
                    "start": {
                        "line": line,
                        "character": character
                    },
                    "end": {
                        "line": line,
                        "character": end_character
                    }
                },
                "severity": 1,
                "source": "kastel",
                "message": diagnostic.message
            })
        })
        .collect::<Vec<_>>();

    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": diagnostics
        }
    })
}

fn diagnostic_end_character(
    source: &str,
    line: usize,
    column: usize,
    start_character: u32,
) -> u32 {
    let line_index =
        line.saturating_sub(1);

    let Some(line_text) =
        source.lines().nth(line_index)
    else {
        return start_character
            .saturating_add(1);
    };

    let byte_index =
        column.saturating_sub(1);

    if byte_index >= line_text.len() {
        return start_character
            .saturating_add(1);
    }

    let Some(character) =
        line_text[byte_index..]
            .chars()
            .next()
    else {
        return start_character
            .saturating_add(1);
    };

    start_character.saturating_add(
        character.len_utf16() as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_diagnostic_spans_one_character() {
        let source =
            "const VALUE = 42\n";

        let end =
            diagnostic_end_character(
                source,
                1,
                7,
                6,
            );

        assert_eq!(
            end,
            7
        );
    }

    #[test]
    fn utf16_diagnostic_spans_surrogate_character() {
        let source =
            "😀abc\n";

        let end =
            diagnostic_end_character(
                source,
                1,
                1,
                0,
            );

        assert_eq!(
            end,
            2
        );
    }

    #[test]
    fn invalid_position_keeps_valid_one_character_range() {
        let source =
            "abc\n";

        let end =
            diagnostic_end_character(
                source,
                10,
                10,
                5,
            );

        assert_eq!(
            end,
            6
        );
    }
}