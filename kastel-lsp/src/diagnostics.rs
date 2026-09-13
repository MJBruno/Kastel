use serde_json::{Value, json};

use crate::analyzer::Diagnostic;
use crate::position::kastel_to_lsp;

pub fn build_diagnostics(uri: &str, source: &str, diagnostics: Vec<Diagnostic>) -> Value {
    let diagnostics = diagnostics
        .into_iter()
        .map(|diagnostic| {
            let (line, character) = kastel_to_lsp(source, diagnostic.line, diagnostic.column);

            json!({
                "range": {
                    "start": {
                        "line": line,
                        "character": character
                    },
                    "end": {
                        "line": line,
                        "character": character + 1
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
