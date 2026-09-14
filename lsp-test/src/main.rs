use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Stdio};

use serde_json::{json, Value};

fn main() {
    let root = create_test_workspace();

    let root_uri = path_to_file_uri(&root);

    let main_path = root.join("main.ks");
    let math_path = root.join("math").join("xx.ks");

    let main_uri = path_to_file_uri(&main_path);
    let math_uri = path_to_file_uri(&math_path);

    let source = "import math.xx\n\
         \n\
         func main() {\n\
             print(VALUE)\n\
         }\n";

    let math_source = "const VALUE = 42\n\
         \n\
         func hello() {\n\
             return VALUE\n\
         }\n";

    fs::write(&main_path, source)
        .expect("failed to write main.ks");

    fs::write(&math_path, math_source)
        .expect("failed to write math/xx.ks");

    let value_position =
        position_of(source, "VALUE");

    println!(
        "VALUE position: line={}, character={}",
        value_position.0,
        value_position.1
    );

    let mut client =
        LspClient::new();

    // ============================================================
    // INITIALIZE
    // ============================================================

    println!("→ initialize");

    let initialize_id =
        client.next_id();

    client.send_request(
        initialize_id,
        "initialize",
        json!({
            "processId": null,
            "rootUri": root_uri,
            "capabilities": {}
        }),
    );

    let response =
        client
            .read_response()
            .expect(
                "missing initialize response",
            );

    assert_eq!(
        response["id"],
        initialize_id
    );

    assert!(
        response["result"]["capabilities"]["hoverProvider"]
            .as_bool()
            .unwrap()
    );

    assert!(
        response["result"]["capabilities"]["definitionProvider"]
            .as_bool()
            .unwrap()
    );

    assert!(
        response["result"]["capabilities"]["referencesProvider"]
            .as_bool()
            .unwrap()
    );

    assert!(
        response["result"]["capabilities"]["renameProvider"]
            .as_bool()
            .unwrap()
    );

    assert!(
        response["result"]["capabilities"]["documentSymbolProvider"]
            .as_bool()
            .unwrap()
    );

    println!("← initialize OK");

    // ============================================================
    // INITIALIZED
    // ============================================================

    println!("→ initialized");

    client.send_notification(
        "initialized",
        json!({}),
    );

    // ============================================================
    // DID OPEN
    // ============================================================

    println!("→ didOpen main.ks");

    client.send_notification(
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": main_uri,
                "languageId": "kastel",
                "version": 1,
                "text": source
            }
        }),
    );

    let diagnostics =
        client
            .read_notification(
                "textDocument/publishDiagnostics",
            )
            .expect(
                "missing diagnostics notification",
            );

    assert_eq!(
        diagnostics["params"]["uri"],
        main_uri
    );

    assert!(
        diagnostics["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty(),
        "valid source unexpectedly produced diagnostics"
    );

    println!("← diagnostics OK");

    // ============================================================
    // DOCUMENT SYMBOL
    // ============================================================

    println!(
        "→ documentSymbol main.ks"
    );

    let symbols_id =
        client.next_id();

    client.send_request(
        symbols_id,
        "textDocument/documentSymbol",
        json!({
            "textDocument": {
                "uri": main_uri
            }
        }),
    );

    let response =
        client
            .read_response()
            .expect(
                "missing documentSymbol response",
            );

    assert_eq!(
        response["id"],
        symbols_id
    );

    let symbols =
        response["result"]
            .as_array()
            .expect(
                "documentSymbol result is not an array",
            );

    assert!(
        symbols.iter().any(
            |symbol| {
                symbol["name"] == "xx"
            }
        )
    );

    assert!(
        symbols.iter().any(
            |symbol| {
                symbol["name"] == "main"
            }
        )
    );

    println!(
        "← documentSymbol main.ks OK"
    );

    // ============================================================
    // IMPORTED DOCUMENT SYMBOLS
    // ============================================================

    println!(
        "→ documentSymbol math/xx.ks"
    );

    let imported_symbols_id =
        client.next_id();

    client.send_request(
        imported_symbols_id,
        "textDocument/documentSymbol",
        json!({
            "textDocument": {
                "uri": math_uri
            }
        }),
    );

    let response =
        client
            .read_response()
            .expect(
                "missing imported documentSymbol response",
            );

    assert_eq!(
        response["id"],
        imported_symbols_id
    );

    let imported_result =
        response["result"]
            .as_array()
            .expect(
                "imported documentSymbol result is not an array",
            );

    assert!(
        imported_result.iter().any(
            |symbol| {
                symbol["name"] == "VALUE"
            }
        )
    );

    assert!(
        imported_result.iter().any(
            |symbol| {
                symbol["name"] == "hello"
            }
        )
    );

    println!(
        "← imported module symbols OK"
    );

    // ============================================================
    // HOVER
    // ============================================================

    println!("→ hover VALUE");

    let hover_id =
        client.next_id();

    client.send_request(
        hover_id,
        "textDocument/hover",
        json!({
            "textDocument": {
                "uri": main_uri
            },
            "position": {
                "line": value_position.0,
                "character": value_position.1
            }
        }),
    );

    let response =
        client
            .read_response()
            .expect(
                "missing hover response",
            );

    assert_eq!(
        response["id"],
        hover_id
    );

    assert!(
        !response["result"].is_null(),
        "hover returned null"
    );

    println!("← hover OK");

    // ============================================================
    // DEFINITION
    // ============================================================

    println!(
        "→ definition VALUE"
    );

    let definition_id =
        client.next_id();

    client.send_request(
        definition_id,
        "textDocument/definition",
        json!({
            "textDocument": {
                "uri": main_uri
            },
            "position": {
                "line": value_position.0,
                "character": value_position.1
            }
        }),
    );

    let response =
        client
            .read_response()
            .expect(
                "missing definition response",
            );

    assert_eq!(
        response["id"],
        definition_id
    );

    let result =
        &response["result"];

    assert!(
        !result.is_null(),
        "definition returned null"
    );

    let found =
        if let Some(location) =
            result.as_object()
        {
            location
                .get("uri")
                .and_then(
                    Value::as_str,
                )
                .map(|uri| {
                    uri == math_uri
                })
                .unwrap_or(false)
        } else if let Some(locations) =
            result.as_array()
        {
            locations.iter().any(
                |location| {
                    location["uri"]
                        == math_uri
                },
            )
        } else {
            panic!(
                "invalid definition result: {}",
                result
            );
        };

    assert!(
        found,
        "VALUE definition does not point to {}: {}",
        math_uri,
        result
    );

    println!("← definition OK");

    // ============================================================
    // REFERENCES
    // ============================================================

    println!("→ references VALUE");

    let references_id =
        client.next_id();

    client.send_request(
        references_id,
        "textDocument/references",
        json!({
            "textDocument": {
                "uri": main_uri
            },
            "position": {
                "line": value_position.0,
                "character": value_position.1
            },
            "context": {
                "includeDeclaration": true
            }
        }),
    );

    let response =
        client
            .read_response()
            .expect(
                "missing references response",
            );

    assert_eq!(
        response["id"],
        references_id
    );

    let references =
        response["result"]
            .as_array()
            .expect(
                "references result is not an array",
            );

    assert!(
        !references.is_empty(),
        "references result is empty"
    );

    let has_main_reference =
        references.iter().any(
            |location| {
                location["uri"]
                    == main_uri
            },
        );

    let has_math_reference =
        references.iter().any(
            |location| {
                location["uri"]
                    == math_uri
            },
        );

    assert!(
        has_main_reference,
        "main.ks reference was not found"
    );

    assert!(
        has_math_reference,
        "math/xx.ks reference was not found"
    );

    println!("← references OK");

    // ============================================================
    // RENAME
    // ============================================================

    println!("→ rename VALUE");

    let rename_id =
        client.next_id();

    client.send_request(
        rename_id,
        "textDocument/rename",
        json!({
            "textDocument": {
                "uri": main_uri
            },
            "position": {
                "line": value_position.0,
                "character": value_position.1
            },
            "newName": "NUMBER"
        }),
    );

    let response =
        client
            .read_response()
            .expect(
                "missing rename response",
            );

    assert_eq!(
        response["id"],
        rename_id
    );

    let changes =
        response["result"]["changes"]
            .as_object()
            .expect(
                "rename result has no changes",
            );

    assert!(
        !changes.is_empty(),
        "rename returned no changes"
    );

    assert!(
        changes.contains_key(
            &main_uri,
        ),
        "rename does not contain main.ks"
    );

    assert!(
        changes.contains_key(
            &math_uri,
        ),
        "rename does not contain math/xx.ks"
    );

    let main_edits =
        changes[&main_uri]
            .as_array()
            .expect(
                "main.ks rename edits are not an array",
            );

    let math_edits =
        changes[&math_uri]
            .as_array()
            .expect(
                "math/xx.ks rename edits are not an array",
            );

    assert!(
        !main_edits.is_empty(),
        "main.ks has no rename edits"
    );

    assert!(
        !math_edits.is_empty(),
        "math/xx.ks has no rename edits"
    );

    assert!(
        main_edits.iter().all(
            |edit| {
                edit["newText"]
                    == "NUMBER"
            }
        )
    );

    assert!(
        math_edits.iter().all(
            |edit| {
                edit["newText"]
                    == "NUMBER"
            }
        )
    );

    println!("← rename OK");

    // ============================================================
    // COMPLETION
    // ============================================================

    println!("→ completion");

    let completion_id =
        client.next_id();

    client.send_request(
        completion_id,
        "textDocument/completion",
        json!({
            "textDocument": {
                "uri": main_uri
            },
            "position": {
                "line": value_position.0,
                "character": value_position.1
            }
        }),
    );

    let response =
        client
            .read_response()
            .expect(
                "missing completion response",
            );

    assert_eq!(
        response["id"],
        completion_id
    );

    let items =
        response["result"]["items"]
            .as_array()
            .expect(
                "completion result has no items",
            );

    assert!(
        items.iter().any(
            |item| {
                item["label"]
                    == "main"
            }
        )
    );

    assert!(
        items.iter().any(
            |item| {
                item["label"]
                    == "func"
            }
        )
    );

    println!("← completion OK");

    // ============================================================
    // DID CHANGE : INTRODUCE SYNTAX ERROR
    // ============================================================

    println!(
        "→ didChange invalid source"
    );

    let invalid_source =
        "import math.xx\n\
         \n\
         func main() {\n\
             print(VALUE)\n\
         \n";

    client.send_notification(
        "textDocument/didChange",
        json!({
            "textDocument": {
                "uri": main_uri,
                "version": 2
            },
            "contentChanges": [
                {
                    "text": invalid_source
                }
            ]
        }),
    );

    let diagnostics =
        client
            .read_notification(
                "textDocument/publishDiagnostics",
            )
            .expect(
                "missing invalid-source diagnostics",
            );

    assert_eq!(
        diagnostics["params"]["uri"],
        main_uri
    );

    let invalid_diagnostics =
        diagnostics["params"]["diagnostics"]
            .as_array()
            .expect(
                "diagnostics is not an array",
            );

    assert!(
        !invalid_diagnostics.is_empty(),
        "invalid Kastel source produced no diagnostics"
    );

    for diagnostic
        in invalid_diagnostics
    {
        assert!(
            diagnostic["message"]
                .as_str()
                .is_some(),
            "diagnostic has no message"
        );

        assert_eq!(
            diagnostic["source"],
            "kastel"
        );

        assert_eq!(
            diagnostic["severity"],
            1
        );

        assert!(
            diagnostic["range"]["start"]["line"]
                .is_number()
        );

        assert!(
            diagnostic["range"]["start"]["character"]
                .is_number()
        );

        assert!(
            diagnostic["range"]["end"]["line"]
                .is_number()
        );

        assert!(
            diagnostic["range"]["end"]["character"]
                .is_number()
        );
    }

    println!(
        "← invalid diagnostics OK"
    );

    // ============================================================
    // DID CHANGE : REPAIR SOURCE
    // ============================================================

    println!(
        "→ didChange valid source"
    );

    let changed_source =
        "import math.xx\n\
         \n\
         func main() {\n\
             print(VALUE)\n\
             print(42)\n\
         }\n";

    client.send_notification(
        "textDocument/didChange",
        json!({
            "textDocument": {
                "uri": main_uri,
                "version": 3
            },
            "contentChanges": [
                {
                    "text": changed_source
                }
            ]
        }),
    );

    let diagnostics =
        client
            .read_notification(
                "textDocument/publishDiagnostics",
            )
            .expect(
                "missing repaired-source diagnostics",
            );

    assert_eq!(
        diagnostics["params"]["uri"],
        main_uri
    );

    assert!(
        diagnostics["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty(),
        "repaired source still has diagnostics"
    );

    println!(
        "← repaired diagnostics OK"
    );

    // ============================================================
    // DID CLOSE
    // ============================================================

    println!("→ didClose");

    client.send_notification(
        "textDocument/didClose",
        json!({
            "textDocument": {
                "uri": main_uri
            }
        }),
    );

    let diagnostics =
        client
            .read_notification(
                "textDocument/publishDiagnostics",
            )
            .expect(
                "missing didClose diagnostics",
            );

    assert_eq!(
        diagnostics["params"]["uri"],
        main_uri
    );

    assert!(
        diagnostics["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    println!("← didClose OK");

    // ============================================================
    // SHUTDOWN
    // ============================================================

    println!("→ shutdown");

    let shutdown_id =
        client.next_id();

    client.send_request(
        shutdown_id,
        "shutdown",
        Value::Null,
    );

    let response =
        client
            .read_response()
            .expect(
                "missing shutdown response",
            );

    assert_eq!(
        response["id"],
        shutdown_id
    );

    assert_eq!(
        response["result"],
        Value::Null
    );

    println!("← shutdown OK");

    // ============================================================
    // EXIT
    // ============================================================

    println!("→ exit");

    client.send_notification_ignore_error(
        "exit",
        Value::Null,
    );

    client.stop();

    println!();
    println!("LSP test OK");

    let _ =
        fs::remove_dir_all(root);
}

fn position_of(
    source: &str,
    needle: &str,
) -> (u32, u32) {
    for (
        line,
        line_text,
    ) in source.lines().enumerate()
    {
        if let Some(character) =
            line_text.find(needle)
        {
            return (
                line as u32,
                character as u32,
            );
        }
    }

    panic!(
        "unable to find `{}` in source",
        needle
    );
}

fn create_test_workspace() -> PathBuf {
    let root =
        std::env::temp_dir()
            .join("kastel-lsp-test");

    let math =
        root.join("math");

    if root.exists() {
        fs::remove_dir_all(&root)
            .expect(
                "failed to remove old test workspace",
            );
    }

    fs::create_dir_all(&math)
        .expect(
            "failed to create test workspace",
        );

    root
}

fn path_to_file_uri(
    path: &PathBuf,
) -> String {
    let absolute =
        fs::canonicalize(path)
            .unwrap_or_else(|_| {
                path.clone()
            });

    let mut path =
        absolute
            .to_string_lossy()
            .to_string();

    #[cfg(windows)]
    {
        if let Some(stripped) =
            path.strip_prefix(r"\\?\")
        {
            path =
                stripped.to_string();
        }
    }

    path =
        path.replace('\\', "/");

    #[cfg(windows)]
    {
        format!(
            "file:///{}",
            path
        )
    }

    #[cfg(not(windows))]
    {
        format!(
            "file://{}",
            path
        )
    }
}

struct LspClient {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    next_id: i64,
}

impl LspClient {
    fn new() -> Self {
        let executable =
            find_kastel_lsp();

        let mut child =
            std::process::Command::new(
                executable,
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect(
                "failed to start kastel-lsp",
            );

        let stdin =
            child
                .stdin
                .take()
                .expect(
                    "failed to open LSP stdin",
                );

        let stdout =
            child
                .stdout
                .take()
                .expect(
                    "failed to open LSP stdout",
                );

        Self {
            child,
            stdin,
            reader:
                BufReader::new(stdout),
            next_id: 1,
        }
    }

    fn next_id(&mut self) -> i64 {
        let id =
            self.next_id;

        self.next_id += 1;

        id
    }

    fn send_request(
        &mut self,
        id: i64,
        method: &str,
        params: Value,
    ) {
        let message =
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": method,
                "params": params
            });

        self.write_message(
            message,
        );
    }

    fn send_notification(
        &mut self,
        method: &str,
        params: Value,
    ) {
        let message =
            json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params
            });

        self.write_message(
            message,
        );
    }

    fn send_notification_ignore_error(
        &mut self,
        method: &str,
        params: Value,
    ) {
        let message =
            json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params
            });

        let _ =
            self.write_message_result(
                message,
            );
    }

    fn write_message(
        &mut self,
        message: Value,
    ) {
        self.write_message_result(
            message,
        )
        .expect(
            "failed to write LSP message",
        );
    }

    fn write_message_result(
        &mut self,
        message: Value,
    ) -> std::io::Result<()> {
        let body =
            serde_json::to_vec(&message)
                .map_err(
                    std::io::Error::other,
                )?;

        write!(
            self.stdin,
            "Content-Length: {}\r\n\r\n",
            body.len()
        )?;

        self.stdin.write_all(
            &body,
        )?;

        self.stdin.flush()?;

        Ok(())
    }

    fn read_message(
        &mut self,
    ) -> Option<Value> {
        let mut content_length =
            None;

        loop {
            let mut line =
                String::new();

            let bytes =
                self.reader
                    .read_line(
                        &mut line,
                    )
                    .ok()?;

            if bytes == 0 {
                return None;
            }

            let line =
                line.trim_end_matches(
                    ['\r', '\n'],
                );

            if line.is_empty() {
                break;
            }

            if let Some(value) =
                line.strip_prefix(
                    "Content-Length:",
                )
            {
                content_length =
                    value
                        .trim()
                        .parse::<usize>()
                        .ok();
            }
        }

        let length =
            content_length?;

        let mut body =
            vec![0u8; length];

        self.reader
            .read_exact(
                &mut body,
            )
            .ok()?;

        serde_json::from_slice(
            &body,
        )
        .ok()
    }

    fn read_response(
        &mut self,
    ) -> Option<Value> {
        loop {
            let message =
                self.read_message()?;

            if message
                .get("id")
                .is_some()
            {
                return Some(
                    message,
                );
            }

            if let Some(method) =
                message
                    .get("method")
                    .and_then(
                        Value::as_str,
                    )
            {
                println!(
                    "← notification: {}",
                    method
                );
            }
        }
    }

    fn read_notification(
        &mut self,
        expected_method: &str,
    ) -> Option<Value> {
        loop {
            let message =
                self.read_message()?;

            if let Some(method) =
                message
                    .get("method")
                    .and_then(
                        Value::as_str,
                    )
            {
                if method
                    == expected_method
                {
                    return Some(
                        message,
                    );
                }
            }
        }
    }

    fn stop(&mut self) {
        let _ =
            self.child.kill();

        let _ =
            self.child.wait();
    }
}

fn find_kastel_lsp() -> PathBuf {
    let root =
        PathBuf::from(
            env!("CARGO_MANIFEST_DIR"),
        )
        .parent()
        .expect(
            "failed to find repository root",
        )
        .to_path_buf();

    #[cfg(windows)]
    let executable =
        root.join("kastel-lsp")
            .join("target")
            .join("debug")
            .join("kastel-lsp.exe");

    #[cfg(not(windows))]
    let executable =
        root.join("kastel-lsp")
            .join("target")
            .join("debug")
            .join("kastel-lsp");

    executable
}