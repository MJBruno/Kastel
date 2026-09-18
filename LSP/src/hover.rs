use serde_json::{Value, json};

use crate::class_index::ClassIndex;
use crate::completion::{
    detect_member_access, find_enclosing_class, infer_class_of_variable, line_and_byte_to_offset,
};
use crate::language::{
    ARRAY_METHODS, BUILTIN_FUNCTIONS, DICT_METHODS, STRING_METHODS, TUPLE_METHODS,
};
use crate::lsp_position::offset_to_lsp;
use crate::symbols::SymbolKind;
use crate::text_util::{is_identifier_byte, utf16_character_to_byte_index};
use crate::workspace::{Workspace, WorkspaceDocument};

pub fn build_hover(workspace: &Workspace, uri: &str, line: u32, character: u32) -> Option<Value> {
    let document = workspace.get(uri)?;

    let line_text = document.text.lines().nth(line as usize)?;

    let byte_index = utf16_character_to_byte_index(line_text, character as usize);

    let bytes = line_text.as_bytes();

    let mut start = byte_index;
    let mut end = byte_index;

    while start > 0 && is_identifier_byte(bytes[start - 1]) {
        start -= 1;
    }

    while end < bytes.len() && is_identifier_byte(bytes[end]) {
        end += 1;
    }

    if start == end {
        return None;
    }

    let word = &line_text[start..end];

    let range = word_range(document, line, start, end);

    /*
     * 1. `X.word` : membre d'une classe connue (this/base/variable
     *    inférée via `= new Classe(...)`), sinon repli sur les
     *    tables génériques de méthodes de collection.
     */
    if let Some(ctx) = detect_member_access(line_text, start) {
        if let Some(value) = build_member_hover(document, line, start, &ctx.path, word, &range) {
            return Some(value);
        }
    }

    /*
     * 2. Fonction/valeur native de la stdlib Kastel.
     */
    if let Some((name, signature, doc)) = BUILTIN_FUNCTIONS.iter().find(|(name, ..)| *name == word)
    {
        return Some(hover_value(
            format!(
                "**{}** _(native)_\n\n```kastel\n{}\n```\n\n{}",
                name, signature, doc
            ),
            range,
        ));
    }

    /*
     * 3. Symbole déclaré dans ce document, sinon ailleurs dans le
     *    workspace (utile pour un symbole importé).
     */
    if let Some(value) = build_symbol_hover(document, word, &range) {
        return Some(value);
    }

    for (other_uri, other_document) in workspace.iter() {
        if other_uri == uri {
            continue;
        }

        if let Some(value) = build_symbol_hover(other_document, word, &range) {
            return Some(value);
        }
    }

    None
}

/// Survol pour `X.word` : champs/méthodes de la classe résolue pour
/// `X`, ou repli sur les tables génériques de méthodes de collection.
fn build_member_hover(
    document: &WorkspaceDocument,
    line: u32,
    word_byte_start: usize,
    base_path: &str,
    word: &str,
    range: &Value,
) -> Option<Value> {
    let offset = line_and_byte_to_offset(&document.text, line as usize, word_byte_start);

    if base_path == "this" || base_path == "base" {
        if let Some(class_name) = find_enclosing_class(&document.text, &document.classes, offset) {
            if let Some(value) = class_member_hover(
                &document.classes,
                &class_name,
                base_path == "this",
                word,
                range,
            ) {
                return Some(value);
            }
        }
    }

    if let Some(class_name) = infer_class_of_variable(&document.text, base_path, &document.classes)
    {
        if let Some(value) = class_member_hover(&document.classes, &class_name, true, word, range) {
            return Some(value);
        }
    }

    for (name, signature, doc) in ARRAY_METHODS
        .iter()
        .chain(STRING_METHODS)
        .chain(DICT_METHODS)
        .chain(TUPLE_METHODS)
    {
        if *name == word {
            return Some(hover_value(
                format!("**{}**\n\n```kastel\n{}\n```\n\n{}", name, signature, doc),
                range.clone(),
            ));
        }
    }

    None
}

/// Survol pour un champ ou une méthode réelle de `class_name`,
/// résolue via `ClassIndex` (déduit de l'AST, y compris via
/// l'héritage).
fn class_member_hover(
    classes: &ClassIndex,
    class_name: &str,
    include_fields: bool,
    word: &str,
    range: &Value,
) -> Option<Value> {
    if include_fields {
        for field in classes.all_fields(class_name) {
            if field == word {
                return Some(hover_value(
                    format!(
                        "**{}**\n\nChamp de `{}` (déduit de `this.{}`)",
                        field, class_name, field
                    ),
                    range.clone(),
                ));
            }
        }
    }

    let methods = if include_fields {
        classes.all_methods(class_name)
    } else {
        classes.base_methods(class_name)
    };

    for method in methods {
        if method.name == word {
            let owner = if include_fields {
                classes
                    .method_owner(class_name, method.name.as_str())
                    .unwrap_or_else(|| class_name.to_string())
            } else {
                classes
                    .base_method_owner(class_name, method.name.as_str())
                    .unwrap_or_else(|| class_name.to_string())
            };

            return Some(hover_value(
                format!(
                    "**{}**\n\n```kastel\nfunc {}({})\n```\n\nMéthode de `{}`",
                    method.name,
                    method.name,
                    method.params.join(", "),
                    owner
                ),
                range.clone(),
            ));
        }
    }

    None
}

/// Survol pour un symbole (variable/fonction/classe/interface/import)
/// déclaré dans `document`. Pour une classe ou une interface, ajoute
/// la liste de ses méthodes, champs et classes de base.
fn build_symbol_hover(document: &WorkspaceDocument, word: &str, range: &Value) -> Option<Value> {
    let symbol = document.symbols.get(word)?;

    let kind_label = match symbol.kind {
        SymbolKind::Variable => "Variable",
        SymbolKind::Function => "Function",
        SymbolKind::Class => "Class",
        SymbolKind::Interface => "Interface",
        SymbolKind::Import => "Import",
    };

    let mut contents = format!("**{}**\n\n{}", symbol.name, kind_label);

    if matches!(symbol.kind, SymbolKind::Class | SymbolKind::Interface) {
        if let Some(info) = document.classes.get(&symbol.name) {
            if !info.bases.is_empty() {
                contents.push_str(&format!("\n\nHérite de : `{}`", info.bases.join(", ")));
            }

            if !info.methods.is_empty() {
                let list = info
                    .methods
                    .iter()
                    .map(|m| format!("- `{}({})`", m.name, m.params.join(", ")))
                    .collect::<Vec<_>>()
                    .join("\n");

                contents.push_str(&format!("\n\n**Méthodes**\n\n{}", list));
            }

            if !info.fields.is_empty() {
                let fields = info
                    .fields
                    .iter()
                    .map(|f| format!("`{}`", f))
                    .collect::<Vec<_>>()
                    .join(", ");

                contents.push_str(&format!("\n\n**Champs** : {}", fields));
            }
        }
    }

    Some(hover_value(contents, range.clone()))
}

fn hover_value(contents: String, range: Value) -> Value {
    json!({
        "contents": {
            "kind": "markdown",
            "value": contents
        },
        "range": range
    })
}

/// Range LSP (en unités UTF-16) du mot `[start, end)` (bytes, dans
/// `line_text` de la ligne `line`), calculée via une conversion en
/// offset global puis `offset_to_lsp` pour rester cohérente avec le
/// reste du LSP.
fn word_range(document: &WorkspaceDocument, line: u32, start: usize, end: usize) -> Value {
    let start_offset = line_and_byte_to_offset(&document.text, line as usize, start);
    let end_offset = line_and_byte_to_offset(&document.text, line as usize, end);

    let (start_line, start_character) = offset_to_lsp(&document.text, start_offset);
    let (end_line, end_character) = offset_to_lsp(&document.text, end_offset);

    json!({
        "start": { "line": start_line, "character": start_character },
        "end": { "line": end_line, "character": end_character }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::workspace::Workspace;

    fn workspace_with(uri: &str, text: &str) -> Workspace {
        let mut workspace = Workspace::new();
        workspace.open(uri.to_string(), 1, text.to_string());
        workspace
    }

    #[test]
    fn hovers_builtin_function() {
        let workspace = workspace_with("file:///main.ks", "println(1);\n");

        let hover = build_hover(&workspace, "file:///main.ks", 0, 2).expect("expected hover");

        let value = hover["contents"]["value"].as_str().unwrap();

        assert!(value.contains("println"));
        assert!(value.contains("native"));
    }

    #[test]
    fn hovers_local_symbol() {
        let workspace = workspace_with("file:///main.ks", "let total = 1;\nprintln(total);\n");

        let hover = build_hover(&workspace, "file:///main.ks", 1, 10).expect("expected hover");

        let value = hover["contents"]["value"].as_str().unwrap();

        assert!(value.contains("total"));
        assert!(value.contains("Variable"));
    }

    #[test]
    fn hovers_class_with_methods_and_fields() {
        let source = "class Point {\nfunc init(x, y) {\nthis.x = x;\nthis.y = y;\n}\n}\n";

        let workspace = workspace_with("file:///main.ks", source);

        let hover = build_hover(&workspace, "file:///main.ks", 0, 7).expect("expected hover");

        let value = hover["contents"]["value"].as_str().unwrap();

        assert!(value.contains("Point"));
        assert!(value.contains("init"));
        assert!(value.contains("`x`"));
        assert!(value.contains("`y`"));
    }

    #[test]
    fn hovers_this_field_inside_method() {
        let source = "class Point {\nfunc init(x, y) {\nthis.x = x;\nreturn this.x;\n}\n}\n";

        let workspace = workspace_with("file:///main.ks", source);

        // Ligne 3 (0-based) : "return this.x;" -> hover sur "x" après "this."
        let hover = build_hover(&workspace, "file:///main.ks", 3, 12).expect("expected hover");

        let value = hover["contents"]["value"].as_str().unwrap();

        assert!(value.contains("Champ de `Point`"));
    }

    #[test]
    fn hovers_inherited_method_via_base() {
        let source = "class Animal {\nfunc speak() {\nreturn 1;\n}\n}\nclass Dog : Animal {\nfunc speak() {\nreturn base.speak();\n}\n}\n";

        let workspace = workspace_with("file:///main.ks", source);

        // Ligne 7 (0-based) : "return base.speak();" -> hover sur "speak"
        let hover = build_hover(&workspace, "file:///main.ks", 7, 14).expect("expected hover");

        let value = hover["contents"]["value"].as_str().unwrap();

        assert!(value.contains("Méthode de `Animal`"));
    }

    #[test]
    fn falls_back_to_generic_array_method() {
        let source = "let arr = [1, 2, 3];\narr.push(4);\n";

        let workspace = workspace_with("file:///main.ks", source);

        let hover = build_hover(&workspace, "file:///main.ks", 1, 6).expect("expected hover");

        let value = hover["contents"]["value"].as_str().unwrap();

        assert!(value.contains("push"));
    }
}
