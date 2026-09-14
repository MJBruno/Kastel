//! Analyse sémantique basique : détecte les identifiants utilisés
//! mais jamais déclarés dans le document courant ou un document du
//! workspace.
//!
//! Volontairement conservateur : on préfère rater une erreur plutôt
//! que produire un faux positif. Voir les limitations dans le README.

use std::collections::HashSet;

use crate::analyzer::Diagnostic;
use crate::language::{BUILTINS, KEYWORDS};
use crate::text_util::is_identifier_byte;
use crate::workspace::Workspace;

struct Ident {
    name: String,
    offset: usize,
}

/// Analyse sémantique du document `uri`.
pub fn analyze(
    workspace: &Workspace,
    uri: &str,
) -> Vec<Diagnostic> {
    let Some(document) = workspace.get(uri)
    else {
        return Vec::new();
    };

    let known = collect_known_names(workspace);

    let masked =
        mask_strings_and_comments(&document.text);

    let identifiers =
        scan_identifiers(&masked);

    let mut diagnostics =
        Vec::new();

    for ident in identifiers {
        if should_skip_identifier(
            &masked,
            &ident,
            &known,
        ) {
            continue;
        }

        let (line, column) =
            offset_to_position(
                &document.text,
                ident.offset,
            );

        diagnostics.push(Diagnostic::error(
            format!(
                "undefined identifier: {}",
                ident.name
            ),
            line,
            column,
            "undefined-identifier",
        ));
    }

    diagnostics
}

/// Rassemble tous les noms connus : tous les documents du workspace.
fn collect_known_names(
    workspace: &Workspace,
) -> HashSet<String> {
    let mut known =
        HashSet::new();

    for (_, document) in workspace.iter() {
        for symbol in document.symbols.iter() {
            known.insert(
                symbol.name.clone(),
            );
        }
    }

    known
}

fn should_skip_identifier(
    masked: &str,
    ident: &Ident,
    known: &HashSet<String>,
) -> bool {
    let name = ident.name.as_str();

    if KEYWORDS.contains(&name) {
        return true;
    }

    if BUILTINS.contains(&name) {
        return true;
    }

    if known.contains(name) {
        return true;
    }

    if is_member_access(
        masked,
        ident.offset,
    ) {
        return true;
    }

    if is_declaration_context(
        masked,
        ident.offset,
    ) {
        return true;
    }

    if is_first_on_line(
        masked,
        ident.offset,
    ) {
        return true;
    }

    false
}

/// Identifiant précédé (en sautant les blancs) par `.`.
fn is_member_access(
    source: &str,
    offset: usize,
) -> bool {
    source[..offset]
        .trim_end()
        .ends_with('.')
}

/// Identifiant précédé d'un mot-clé de déclaration.
fn is_declaration_context(
    source: &str,
    offset: usize,
) -> bool {
    let trimmed =
        source[..offset].trim_end();

    if trimmed.is_empty() {
        return false;
    }

    let last_word =
        trimmed
            .rsplit(|c: char| {
                !c.is_ascii_alphanumeric()
                    && c != '_'
            })
            .next()
            .unwrap_or("");

    matches!(
        last_word,
        "let"
            | "const"
            | "func"
            | "class"
            | "interface"
            | "import"
            | "from"
            | "export"
            | "as"
    )
}

/// Premier identifiant de la ligne (précédé uniquement de blancs).
fn is_first_on_line(
    source: &str,
    offset: usize,
) -> bool {
    let line_start =
        source[..offset]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);

    source[line_start..offset]
        .chars()
        .all(|c| c.is_whitespace())
}

/// Remplace par des espaces tous les bytes à l'intérieur des chaînes
/// et des commentaires. Préserve la longueur et les `\n` pour que les
/// offsets restent valides.
fn mask_strings_and_comments(
    source: &str,
) -> String {
    let bytes =
        source.as_bytes();

    let mut result =
        bytes.to_vec();

    let mut i =
        0;

    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                let quote = bytes[i];
                result[i] = b' ';
                i += 1;
                while i < bytes.len()
                    && bytes[i] != quote
                {
                    if bytes[i] == b'\\'
                        && i + 1 < bytes.len()
                    {
                        result[i] = b' ';
                        result[i + 1] = b' ';
                        i += 2;
                    } else {
                        if bytes[i] != b'\n' {
                            result[i] = b' ';
                        }
                        i += 1;
                    }
                }
                if i < bytes.len() {
                    result[i] = b' ';
                    i += 1;
                }
            }
            b'/' if i + 1 < bytes.len()
                && bytes[i + 1] == b'/' =>
            {
                result[i] = b' ';
                result[i + 1] = b' ';
                i += 2;
                while i < bytes.len()
                    && bytes[i] != b'\n'
                {
                    result[i] = b' ';
                    i += 1;
                }
            }
            b'/' if i + 1 < bytes.len()
                && bytes[i + 1] == b'*' =>
            {
                result[i] = b' ';
                result[i + 1] = b' ';
                i += 2;
                while i + 1 < bytes.len()
                    && !(bytes[i] == b'*'
                        && bytes[i + 1] == b'/')
                {
                    if bytes[i] != b'\n' {
                        result[i] = b' ';
                    }
                    i += 1;
                }
                if i + 1 < bytes.len() {
                    result[i] = b' ';
                    result[i + 1] = b' ';
                    i += 2;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    String::from_utf8(result)
        .unwrap_or_else(|_| source.to_string())
}

fn scan_identifiers(
    source: &str,
) -> Vec<Ident> {
    let bytes =
        source.as_bytes();

    let mut ids =
        Vec::new();

    let mut i =
        0;

    while i < bytes.len() {
        if is_identifier_start(bytes[i]) {
            let start = i;
            while i < bytes.len()
                && is_identifier_byte(bytes[i])
            {
                i += 1;
            }
            let name =
                std::str::from_utf8(
                    &bytes[start..i],
                )
                .unwrap_or("")
                .to_string();
            ids.push(Ident { name, offset: start });
        } else {
            i += 1;
        }
    }

    ids
}

fn is_identifier_start(
    byte: u8,
) -> bool {
    byte.is_ascii_alphabetic()
        || byte == b'_'
}

/// Convertit un offset byte en position 1-based (ligne, colonne char).
fn offset_to_position(
    source: &str,
    offset: usize,
) -> (usize, usize) {
    let mut line = 1usize;
    let mut line_start = 0usize;

    for (i, c) in source.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            line_start = i + 1;
        }
    }

    let column =
        source[line_start..offset]
            .chars()
            .count()
            + 1;

    (line, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace_with(text: &str) -> Workspace {
        let mut workspace = Workspace::new();
        workspace.open(
            "file:///main.ks".to_string(),
            1,
            text.to_string(),
        );
        workspace
    }

    #[test]
    fn detects_undefined_identifier() {
        let source = r#"println("TEST ARRAY METHODS");

let values = [1, 2, 3];
println(val.length)
"#;

        let workspace = workspace_with(source);
        let diagnostics =
            analyze(&workspace, "file:///main.ks");

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("val"));
        assert_eq!(
            diagnostics[0].code.as_deref(),
            Some("undefined-identifier")
        );
        assert_eq!(diagnostics[0].severity, 1);
        assert_eq!(diagnostics[0].line, 4);
    }

    #[test]
    fn allows_declared_identifier() {
        let source = r#"let values = [1, 2, 3];
println(values.length)
"#;

        let workspace = workspace_with(source);
        let diagnostics =
            analyze(&workspace, "file:///main.ks");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn allows_builtins() {
        let source =
            "println(\"hello\")\n";

        let workspace = workspace_with(source);
        let diagnostics =
            analyze(&workspace, "file:///main.ks");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_string_contents() {
        let source =
            "let x = \"val\"\n";

        let workspace = workspace_with(source);
        let diagnostics =
            analyze(&workspace, "file:///main.ks");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_line_comments() {
        let source =
            "// val\nlet x = 1\n";

        let workspace = workspace_with(source);
        let diagnostics =
            analyze(&workspace, "file:///main.ks");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_block_comments() {
        let source =
            "/* val\n   val */\nlet x = 1\n";

        let workspace = workspace_with(source);
        let diagnostics =
            analyze(&workspace, "file:///main.ks");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_member_access() {
        let source = r#"let values = [1, 2, 3];
println(values.length)
"#;

        let workspace = workspace_with(source);
        let diagnostics =
            analyze(&workspace, "file:///main.ks");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_unknown_uri() {
        let workspace = Workspace::new();
        let diagnostics =
            analyze(&workspace, "file:///missing.ks");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_undefined_at_correct_column() {
        let source =
            "println(val)\n";

        let workspace = workspace_with(source);
        let diagnostics =
            analyze(&workspace, "file:///main.ks");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, 1);
        assert_eq!(diagnostics[0].column, 9);
    }
}