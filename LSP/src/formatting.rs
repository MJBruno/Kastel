//! Formateur Kastel : `textDocument/formatting`.
//!
//! Portée volontairement prudente : c'est un **ré-indenteur** basé
//! sur la profondeur d'accolades/parenthèses/crochets, pas un
//! joli-imprimeur (« pretty-printer ») complet reconstruit depuis
//! l'AST. Ce choix est délibéré :
//!
//!   - un pretty-printer AST->source perdrait tous les commentaires
//!     (le lexer Kastel les jette, ils ne survivent pas dans l'AST) ;
//!   - un ré-indenteur ligne par ligne, lui, ne touche jamais au
//!     contenu réel d'une ligne (seulement à l'espace de tête), donc
//!     il ne peut pas corrompre une chaîne, un commentaire ou un
//!     format `{}` — le risque de casser du code est quasi nul.
//!
//! Ce que fait le formateur :
//!   - ré-indente chaque ligne selon la profondeur de `{ } ( ) [ ]`
//!     (en ignorant ce qui est à l'intérieur des chaînes/commentaires) ;
//!   - respecte `tabSize` / `insertSpaces` envoyés par l'éditeur ;
//!   - retire les espaces de fin de ligne ;
//!   - réduit les lignes vides consécutives à une seule ;
//!   - retire les lignes vides en tête/fin de fichier ;
//!   - garantit exactement un `\n` final.
//!
//! Ce qu'il NE fait PAS (hors-scope, pour rester sûr) :
//!   - respacer les opérateurs/virgules à l'intérieur d'une ligne ;
//!   - aligner les commentaires ou réordonner quoi que ce soit.

use serde_json::{json, Value};

use crate::lsp_position::offset_to_lsp;
use crate::text_util::mask_strings_and_comments;
use crate::workspace::Workspace;

/// Construit la liste de `TextEdit` LSP pour reformater tout le
/// document `uri`. `None` si le document n'existe pas.
pub fn build_formatting(
    workspace: &Workspace,
    uri: &str,
    tab_size: u32,
    insert_spaces: bool,
) -> Option<Value> {
    let document = workspace.get(uri)?;

    let formatted = format_source(&document.text, tab_size, insert_spaces);

    if formatted == document.text {
        return Some(Value::Array(Vec::new()));
    }

    let (end_line, end_character) = offset_to_lsp(&document.text, document.text.len());

    Some(json!([
        {
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": end_line, "character": end_character }
            },
            "newText": formatted
        }
    ]))
}

/// Reformate une source Kastel complète. Fonction pure, testable
/// indépendamment du protocole LSP.
pub fn format_source(source: &str, tab_size: u32, insert_spaces: bool) -> String {
    let indent_unit = if insert_spaces {
        " ".repeat(tab_size.max(1) as usize)
    } else {
        "\t".to_string()
    };

    let normalized = source.replace("\r\n", "\n").replace('\r', "\n");

    let masked = mask_strings_and_comments(&normalized);

    let raw_lines: Vec<&str> = normalized.split('\n').collect();
    let masked_lines: Vec<&str> = masked.split('\n').collect();

    debug_assert_eq!(raw_lines.len(), masked_lines.len());

    let mut depth: i64 = 0;
    let mut output: Vec<String> = Vec::with_capacity(raw_lines.len());

    for (raw_line, masked_line) in raw_lines.iter().zip(masked_lines.iter()) {
        let leading_ws = raw_line.len() - raw_line.trim_start().len();
        let trailing_ws = raw_line.len() - raw_line.trim_end().len();

        if leading_ws + trailing_ws >= raw_line.len() {
            // Ligne vide (ou uniquement des espaces) : préservée
            // telle quelle, sans indentation — la passe de
            // compactage des lignes vides s'en occupera ensuite.
            output.push(String::new());
            continue;
        }

        let raw_trimmed = &raw_line[leading_ws..raw_line.len() - trailing_ws];
        let masked_trimmed = &masked_line[leading_ws..masked_line.len() - trailing_ws];

        let leading_closers = count_leading_closers(masked_trimmed);

        let print_depth = (depth - leading_closers as i64).max(0);

        let mut line = String::with_capacity(raw_trimmed.len() + print_depth as usize * indent_unit.len());

        for _ in 0..print_depth {
            line.push_str(&indent_unit);
        }

        line.push_str(raw_trimmed);

        output.push(line);

        depth = (depth + bracket_delta(masked_trimmed)).max(0);
    }

    let output = collapse_blank_lines(output);

    let mut result = output.join("\n");

    if !result.is_empty() {
        result.push('\n');
    }

    result
}

/// Compte les fermetures (`}`, `)`, `]`) consécutives en tête de
/// ligne. S'arrête au premier caractère qui n'en est pas une —
/// donc `"} else {"` renvoie 1, pas 2.
fn count_leading_closers(masked_trimmed: &str) -> usize {
    masked_trimmed
        .chars()
        .take_while(|c| matches!(c, '}' | ')' | ']'))
        .count()
}

/// Delta net d'ouvertures/fermetures sur toute la ligne (masquée).
fn bracket_delta(masked_trimmed: &str) -> i64 {
    let mut delta = 0i64;

    for c in masked_trimmed.chars() {
        match c {
            '{' | '(' | '[' => delta += 1,
            '}' | ')' | ']' => delta -= 1,
            _ => {}
        }
    }

    delta
}

/// Réduit les lignes vides consécutives à une seule, et retire les
/// lignes vides en tête/fin de fichier.
fn collapse_blank_lines(lines: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(lines.len());

    for line in lines {
        let is_blank = line.trim().is_empty();

        if is_blank && out.last().is_some_and(|previous: &String| previous.trim().is_empty()) {
            continue;
        }

        out.push(line);
    }

    while out.first().is_some_and(|line| line.trim().is_empty()) {
        out.remove(0);
    }

    while out.last().is_some_and(|line| line.trim().is_empty()) {
        out.pop();
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reindents_nested_blocks() {
        let source = "func add(a, b) {\nreturn a + b;\n}\n";

        let formatted = format_source(source, 4, true);

        assert_eq!(formatted, "func add(a, b) {\n    return a + b;\n}\n");
    }

    #[test]
    fn handles_else_dedent_then_reindent() {
        let source = "if x {\nfoo();\n} else {\nbar();\n}\n";

        let formatted = format_source(source, 4, true);

        assert_eq!(
            formatted,
            "if x {\n    foo();\n} else {\n    bar();\n}\n"
        );
    }

    #[test]
    fn ignores_braces_inside_strings() {
        let source = "func f() {\nlet s = \"{ not a brace }\";\nreturn s;\n}\n";

        let formatted = format_source(source, 4, true);

        assert_eq!(
            formatted,
            "func f() {\n    let s = \"{ not a brace }\";\n    return s;\n}\n"
        );
    }

    #[test]
    fn ignores_braces_inside_comments() {
        let source = "func f() {\n// { comment }\nreturn 1;\n}\n";

        let formatted = format_source(source, 4, true);

        assert_eq!(
            formatted,
            "func f() {\n    // { comment }\n    return 1;\n}\n"
        );
    }

    #[test]
    fn collapses_multiple_blank_lines() {
        let source = "let a = 1;\n\n\n\nlet b = 2;\n";

        let formatted = format_source(source, 4, true);

        assert_eq!(formatted, "let a = 1;\n\nlet b = 2;\n");
    }

    #[test]
    fn trims_leading_and_trailing_blank_lines() {
        let source = "\n\nlet a = 1;\n\n\n";

        let formatted = format_source(source, 4, true);

        assert_eq!(formatted, "let a = 1;\n");
    }

    #[test]
    fn trims_trailing_whitespace() {
        let source = "let a = 1;   \nlet b = 2;\t\n";

        let formatted = format_source(source, 4, true);

        assert_eq!(formatted, "let a = 1;\nlet b = 2;\n");
    }

    #[test]
    fn respects_tab_size_and_insert_spaces() {
        let source = "class Point {\nfunc init() {\nreturn 1;\n}\n}\n";

        let two_spaces = format_source(source, 2, true);

        assert!(two_spaces.contains("\n  func init() {\n"));
        assert!(two_spaces.contains("\n    return 1;\n"));

        let tabs = format_source(source, 4, false);

        assert!(tabs.contains("\n\tfunc init() {\n"));
        assert!(tabs.contains("\n\t\treturn 1;\n"));
    }

    #[test]
    fn idempotent_on_already_formatted_source() {
        let source = "func add(a, b) {\n    return a + b;\n}\n";

        let formatted = format_source(source, 4, true);

        assert_eq!(formatted, source);
    }

    #[test]
    fn handles_nested_class_and_method() {
        let source =
            "class Point {\nfunc init(x, y) {\nthis.x = x;\nthis.y = y;\n}\nfunc add(other) {\nreturn new Point(this.x + other.x, this.y + other.y);\n}\n}\n";

        let formatted = format_source(source, 4, true);

        let expected = "class Point {\n    func init(x, y) {\n        this.x = x;\n        this.y = y;\n    }\n    func add(other) {\n        return new Point(this.x + other.x, this.y + other.y);\n    }\n}\n";

        assert_eq!(formatted, expected);
    }
}
