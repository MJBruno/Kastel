use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde_json::{Value, json};

use crate::class_index::ClassIndex;
use crate::language::{ARRAY_METHODS, BUILTIN_FUNCTIONS, DICT_METHODS, KEYWORDS, STRING_METHODS, TUPLE_METHODS};
use crate::module_resolver::ModuleResolver;
use crate::symbols::{Symbol, SymbolIndex, SymbolKind};
use crate::text_util::{current_prefix, utf16_character_to_byte_index};
use crate::uri_util::{path_to_uri, uri_to_path};
use crate::workspace::Workspace;

/// Kind LSP utilisé pour un module proposé en complétion.
const KIND_MODULE: u32 = 9;
/// Kind LSP "Method".
const KIND_METHOD: u32 = 2;
/// Kind LSP "Function".
const KIND_FUNCTION: u32 = 3;
/// Kind LSP "Field".
const KIND_FIELD: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    Local,
    Imported,
}

/// Contexte détecté quand le curseur est après un `.`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct MemberContext {
    /// La chaîne avant le `.`, ex. `"math"`, `"math.xx"`, `"xx"`.
    pub(crate) path: String,
    /// Vrai si on est sur une ligne `import ...`.
    pub(crate) is_import_line: bool,
}

pub fn build_completion(
    workspace: &Workspace,
    uri: &str,
    line: u32,
    character: u32,
) -> Option<Value> {
    let document = workspace.get(uri)?;

    let line_text = document.text.lines().nth(line as usize).unwrap_or("");

    let byte_index = utf16_character_to_byte_index(line_text, character as usize);

    /*
     * 1. Contexte « member access » : `X.`
     *
     * On retourne uniquement les items pertinents pour X
     * (sous-modules ou symboles exportés), sans keywords.
     */
    if let Some(ctx) = detect_member_access(line_text, byte_index) {
        let offset = line_and_byte_to_offset(&document.text, line as usize, byte_index);

        let items =
            build_member_completions(workspace, uri, &ctx.path, ctx.is_import_line, offset);

        return Some(json!({
            "isIncomplete": false,
            "items": items
        }));
    }

    /*
     * 2. Complétion classique (préfixe d'identifiant).
     */
    let prefix = current_prefix(line_text, byte_index);

    let mut items = Vec::new();
    let mut seen = HashSet::<String>::new();

    add_keyword_completions(prefix, &mut items, &mut seen);

    add_builtin_completions(prefix, &mut items, &mut seen);

    add_document_completions(
        &document.symbols,
        prefix,
        Scope::Local,
        &mut items,
        &mut seen,
    );

    add_import_completions(workspace, uri, prefix, &mut items, &mut seen);

    Some(json!({
        "isIncomplete": false,
        "items": items
    }))
}

/// Détecte `X.` juste avant le curseur.
///
/// Retourne le chemin `X` (autorisant les `.` internes pour gérer
/// `math.xx.`) et si on est sur une ligne `import ...`.
pub(crate) fn detect_member_access(line: &str, byte_index: usize) -> Option<MemberContext> {
    let before = line.get(..byte_index)?;

    let trimmed = before.trim_end();

    if !trimmed.ends_with('.') {
        return None;
    }

    let head = trimmed[..trimmed.len() - 1].trim_end();

    if head.is_empty() {
        return None;
    }

    // Extrait le chemin final : alphanumériques, `_` et `.`.
    let mut start = head.len();

    for (i, c) in head.char_indices().rev() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
            start = i;
        } else {
            break;
        }
    }

    let path = head[start..].trim_start_matches('.');

    if path.is_empty() {
        return None;
    }

    Some(MemberContext {
        path: path.to_string(),
        is_import_line: line.trim_start().starts_with("import "),
    })
}

/// Convertit une position (ligne 0-based, offset byte dans la ligne)
/// en offset byte global dans `text`.
pub(crate) fn line_and_byte_to_offset(text: &str, line: usize, byte_in_line: usize) -> usize {
    let mut offset = 0usize;

    for (index, raw_line) in text.split('\n').enumerate() {
        if index == line {
            return offset + byte_in_line.min(raw_line.len());
        }

        offset += raw_line.len() + 1;
    }

    text.len()
}

/// Construit les items pour un contexte `X.`.
fn build_member_completions(
    workspace: &Workspace,
    uri: &str,
    path: &str,
    is_import_line: bool,
    offset: usize,
) -> Vec<Value> {
    let Some(document) = workspace.get(uri) else {
        return Vec::new();
    };

    let mut items = Vec::new();
    let mut seen = HashSet::<String>::new();

    /*
     * 0. `this.` / `base.` à l'intérieur d'une méthode de classe :
     *    complète avec les champs/méthodes réellement disponibles,
     *    déduits de l'AST (`ClassIndex`), pas d'une heuristique de
     *    liste générique.
     */
    if !is_import_line && (path == "this" || path == "base") {
        if let Some(class_name) = find_enclosing_class(&document.text, &document.classes, offset)
        {
            add_class_member_completions(
                &document.classes,
                &class_name,
                path == "this",
                &mut items,
                &mut seen,
            );
        }
    }

    /*
     * 0bis. `variable.` où `variable` a été assignée via
     *    `variable = new ClassName(...)` plus haut dans le document :
     *    complète avec les champs/méthodes de `ClassName`.
     */
    if !is_import_line && items.is_empty() && path != "this" && path != "base" {
        if let Some(class_name) =
            infer_class_of_variable(&document.text, path, &document.classes)
        {
            add_class_member_completions(
                &document.classes,
                &class_name,
                true,
                &mut items,
                &mut seen,
            );
        }
    }

    /*
     * 1. Ligne `import X.` : on scanne le système de fichiers
     *    pour proposer les sous-modules suivants.
     */
    if is_import_line {
        if let Some(current_file) = uri_to_path(uri) {
            let current_file = current_file.canonicalize().unwrap_or(current_file);

            let path_parts = path.split('.').map(String::from).collect::<Vec<_>>();

            for name in list_submodules(&current_file, &path_parts) {
                if seen.insert(name.clone()) {
                    items.push(json!({
                        "label": name,
                        "kind": KIND_MODULE,
                        "detail": "Module",
                        "insertText": name,
                        "sortText": format!("0_{}", name)
                    }));
                }
            }
        }
    }

    /*
     * 2. Si `X` correspond à un import (chemin complet ou alias),
     *    on propose les symboles exportés du module.
     *
     *    Ex. :
     *    import math.xx
     *    - `math.xx.` → symbols de xx.ks
     *    - `xx.`      → symbols de xx.ks (alias)
     */
    let imports = parse_imports(&document.text);

    let current_file = uri_to_path(uri);

    let resolver = ModuleResolver::new(None);

    for import in &imports {
        let full = import.parts.join(".");

        let alias = import.parts.last().map(String::as_str).unwrap_or("");

        if path != full && path != alias {
            continue;
        }

        let Some(current_file) = current_file.as_deref() else {
            continue;
        };

        let Some(module_path) = resolver.resolve(current_file, &import.parts) else {
            continue;
        };

        let module_uri = path_to_uri(&module_path);

        let Some(module_document) = workspace.get(&module_uri) else {
            continue;
        };

        add_document_completions(
            &module_document.symbols,
            "",
            Scope::Imported,
            &mut items,
            &mut seen,
        );
    }

    /*
     * 3. Repli générique : si rien de plus précis n'a matché (pas un
     *    import, pas `this`/`base`, pas une variable dont on a pu
     *    déduire la classe), on propose l'union des méthodes des
     *    types de collection (array/string/dict/tuple). C'est moins
     *    précis qu'une vraie inférence de types, mais nettement plus
     *    utile que de ne rien proposer du tout.
     */
    if !is_import_line && items.is_empty() {
        add_generic_collection_completions(&mut items, &mut seen);
    }

    items
}

/// Complète avec les champs (si `include_fields`) et les méthodes
/// d'une classe (et de ses classes de base connues dans ce document),
/// déduits de l'AST via `ClassIndex`.
fn add_class_member_completions(
    classes: &ClassIndex,
    class_name: &str,
    include_fields: bool,
    items: &mut Vec<Value>,
    seen: &mut HashSet<String>,
) {
    if include_fields {
        for field in classes.all_fields(class_name) {
            if !seen.insert(field.to_string()) {
                continue;
            }

            items.push(json!({
                "label": field,
                "kind": KIND_FIELD,
                "detail": format!("this.{}", field),
                "insertText": field,
                "sortText": format!("0_{}", field)
            }));
        }
    }

    let methods = if include_fields {
        classes.all_methods(class_name)
    } else {
        // `base.` : uniquement les méthodes héritées, pas les
        // siennes propres (qui se re-définiraient elles-mêmes).
        classes.base_methods(class_name)
    };

    for method in methods {
        if !seen.insert(method.name.clone()) {
            continue;
        }

        items.push(json!({
            "label": method.name,
            "kind": KIND_METHOD,
            "detail": format!("{}({})", method.name, method.params.join(", ")),
            "insertText": format!("{}($0)", method.name),
            "insertTextFormat": 2,
            "sortText": format!("0_{}", method.name)
        }));
    }
}

/// Union dédupliquée des méthodes array/string/dict/tuple, utilisée
/// en repli quand on ne peut rien inférer de plus précis sur `X.`.
fn add_generic_collection_completions(items: &mut Vec<Value>, seen: &mut HashSet<String>) {
    for (name, signature, doc) in ARRAY_METHODS
        .iter()
        .chain(STRING_METHODS)
        .chain(DICT_METHODS)
        .chain(TUPLE_METHODS)
    {
        if !seen.insert((*name).to_string()) {
            continue;
        }

        items.push(method_completion_item(name, signature, doc, "5_"));
    }
}

/// Trouve le nom de la classe/interface qui englobe directement
/// `offset` dans `text`, en suivant la profondeur d'accolades (chaînes
/// et commentaires masqués). `None` si le curseur n'est dans aucune
/// classe, ou si aucune classe ne matche dans `classes`.
pub(crate) fn find_enclosing_class(text: &str, classes: &ClassIndex, offset: usize) -> Option<String> {
    let offset = offset.min(text.len());

    let masked = crate::text_util::mask_strings_and_comments(&text[..offset]);

    let mut stack: Vec<Option<String>> = Vec::new();
    let mut pending: Option<String> = None;

    let mut chars = masked.char_indices().peekable();

    while let Some((_, c)) = chars.next() {
        if c.is_ascii_alphabetic() || c == '_' {
            let mut word = String::new();
            word.push(c);

            while let Some((_, next_c)) = chars.peek().copied() {
                if next_c.is_ascii_alphanumeric() || next_c == '_' {
                    word.push(next_c);
                    chars.next();
                } else {
                    break;
                }
            }

            if word == "class" || word == "interface" {
                while let Some((_, next_c)) = chars.peek().copied() {
                    if next_c.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }

                let mut name = String::new();

                while let Some((_, next_c)) = chars.peek().copied() {
                    if next_c.is_ascii_alphanumeric() || next_c == '_' {
                        name.push(next_c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                if !name.is_empty() && classes.contains(&name) {
                    pending = Some(name);
                }
            }

            continue;
        }

        match c {
            '{' => stack.push(pending.take()),
            '}' => {
                stack.pop();
            }
            _ => {}
        }
    }

    stack.into_iter().rev().find_map(|entry| entry)
}

/// Cherche, n'importe où dans `text`, une affectation
/// `variable = new ClassName(` (avec ou sans `let`/`const` devant),
/// et renvoie `ClassName` si elle est connue de `classes`. Heuristique
/// textuelle volontairement simple (pas de véritable inférence de
/// types) — première correspondance trouvée dans le document.
pub(crate) fn infer_class_of_variable(text: &str, variable: &str, classes: &ClassIndex) -> Option<String> {
    if variable.is_empty() {
        return None;
    }

    let masked = crate::text_util::mask_strings_and_comments(text);

    let mut search_from = 0usize;

    while let Some(relative) = masked[search_from..].find(variable) {
        let start = search_from + relative;
        let end = start + variable.len();

        let before_is_identifier = masked[..start]
            .chars()
            .next_back()
            .is_some_and(crate::text_util::is_identifier_char);

        let after_is_identifier = masked[end..]
            .chars()
            .next()
            .is_some_and(crate::text_util::is_identifier_char);

        if !before_is_identifier && !after_is_identifier {
            let rest = masked[end..].trim_start();

            if let Some(rest) = rest.strip_prefix('=') {
                if !rest.starts_with('=') {
                    let rest = rest.trim_start();

                    if let Some(rest) = rest.strip_prefix("new") {
                        let rest = rest.trim_start();

                        let name: String = rest
                            .chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                            .collect();

                        if !name.is_empty() && classes.contains(&name) {
                            return Some(name);
                        }
                    }
                }
            }
        }

        search_from = end.max(start + 1);
    }

    None
}

/// Liste les sous-modules disponibles sous `dir/part1/part2/...`.
///
/// Propose à la fois les sous-répertoires et les fichiers `.ks`
/// (sans extension).
fn list_submodules(current_file: &Path, path_parts: &[String]) -> Vec<String> {
    let Some(mut dir) = current_file.parent().map(|p| p.to_path_buf()) else {
        return Vec::new();
    };

    for part in path_parts {
        if part.is_empty() || part == "." || part == ".." {
            return Vec::new();
        }

        dir.push(part);
    }

    if !dir.is_dir() {
        return Vec::new();
    }

    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut names = Vec::<String>::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                names.push(name.to_string());
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("ks") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }

    names.sort();
    names.dedup();
    names
}

fn add_keyword_completions(prefix: &str, items: &mut Vec<Value>, seen: &mut HashSet<String>) {
    for keyword in KEYWORDS {
        if !keyword.starts_with(prefix) {
            continue;
        }

        if !seen.insert((*keyword).to_string()) {
            continue;
        }

        items.push(keyword_completion(keyword));
    }
}

fn keyword_completion(keyword: &str) -> Value {
    match keyword_snippet(keyword) {
        Some(snippet) => json!({
            "label": keyword,
            "kind": 15,
            "detail": "Kastel snippet",
            "insertText": snippet,
            "insertTextFormat": 2,
            "sortText": format!("2_{}", keyword)
        }),
        None => json!({
            "label": keyword,
            "kind": 14,
            "detail": "Kastel keyword",
            "insertText": keyword,
            "sortText": format!("2_{}", keyword)
        }),
    }
}

fn keyword_snippet(keyword: &str) -> Option<&'static str> {
    Some(match keyword {
        "const" => "const ${1:NAME} = ${0}",
        "let" => "let ${1:name} = ${0}",
        "func" => "func ${1:name}(${2}) {\n\t${0}\n}",
        "class" => "class ${1:Name} {\n\t${0}\n}",
        "interface" => "interface ${1:Name} {\n\t${0}\n}",
        "if" => "if ${1:condition} {\n\t${0}\n}",
        "else" => "else {\n\t${0}\n}",
        "while" => "while ${1:condition} {\n\t${0}\n}",
        "for" => "for ${1:item} in ${2:collection} {\n\t${0}\n}",
        "import" => "import ${1:module.path}",
        "export" => "export ${0}",
        "try" => "try {\n\t${0}\n} catch ${1:err} {\n\t\n}",
        "catch" => "catch ${1:err} {\n\t${0}\n}",
        "finally" => "finally {\n\t${0}\n}",
        "throw" => "throw ${0}",
        "new" => "new ${1:ClassName}(${0})",
        "match" => "match ${1:value} {\n\t${0}\n}",
        "return" => "return ${0}",
        _ => return None,
    })
}

/// Complétion des fonctions/valeurs globales de la stdlib Kastel
/// (`println`, `format`, `range`, `abs`, ...). Sans ça, aucune
/// fonction native ne se propose jamais — seuls les symboles
/// déclarés par l'utilisateur (via `SymbolIndex`) le sont.
fn add_builtin_completions(prefix: &str, items: &mut Vec<Value>, seen: &mut HashSet<String>) {
    for (name, signature, doc) in BUILTIN_FUNCTIONS {
        if !name.starts_with(prefix) {
            continue;
        }

        if !seen.insert((*name).to_string()) {
            continue;
        }

        items.push(json!({
            "label": *name,
            "kind": KIND_FUNCTION,
            "detail": *signature,
            "documentation": {
                "kind": "markdown",
                "value": *doc
            },
            "insertText": format!("{}($0)", name),
            "insertTextFormat": 2,
            "sortText": format!("1_{}", name)
        }));
    }
}

/// Complétion d'une méthode/propriété de collection (`array`,
/// `string`, `dict`, `tuple`) : `(nom, signature, doc)` -> item LSP.
fn method_completion_item(name: &str, signature: &str, doc: &str, sort_prefix: &str) -> Value {
    // `length` est une propriété sur array/tuple : pas de `()`
    // insérée automatiquement pour ne pas induire en erreur.
    let insert_text = if name == "length" {
        name.to_string()
    } else {
        format!("{}($0)", name)
    };

    json!({
        "label": name,
        "kind": KIND_METHOD,
        "detail": signature,
        "documentation": {
            "kind": "markdown",
            "value": doc
        },
        "insertText": insert_text,
        "insertTextFormat": 2,
        "sortText": format!("{}{}", sort_prefix, name)
    })
}

fn add_document_completions(
    symbols: &SymbolIndex,
    prefix: &str,
    scope: Scope,
    items: &mut Vec<Value>,
    seen: &mut HashSet<String>,
) {
    for symbol in symbols.iter() {
        if scope == Scope::Imported && !symbol.is_exported {
            continue;
        }

        if !symbol.name.starts_with(prefix) {
            continue;
        }

        if !seen.insert(symbol.name.clone()) {
            continue;
        }

        items.push(symbol_completion(symbol, scope));
    }
}

fn symbol_completion(symbol: &Symbol, scope: Scope) -> Value {
    let (sort_prefix, detail_prefix) = match scope {
        Scope::Local => ("0_", ""),
        Scope::Imported => ("1_", "Imported "),
    };

    json!({
        "label": symbol.name,
        "kind": completion_kind(symbol.kind),
        "detail": format!(
            "{}{}",
            detail_prefix,
            completion_detail(symbol.kind)
        ),
        "insertText": symbol.name,
        "sortText": format!(
            "{}{}",
            sort_prefix,
            symbol.name
        )
    })
}

fn add_import_completions(
    workspace: &Workspace,
    uri: &str,
    prefix: &str,
    items: &mut Vec<Value>,
    seen: &mut HashSet<String>,
) {
    let Some(document) = workspace.get(uri) else {
        return;
    };

    let current_file = uri_to_path(uri);

    let resolver = ModuleResolver::new(None);

    let imports = parse_imports(&document.text);

    for import in imports {
        let Some(current_file) = current_file.as_deref() else {
            continue;
        };

        let Some(module_path) = resolver.resolve(current_file, &import.parts) else {
            continue;
        };

        let module_uri = path_to_uri(&module_path);

        let Some(module_document) = workspace.get(&module_uri) else {
            continue;
        };

        add_document_completions(
            &module_document.symbols,
            prefix,
            Scope::Imported,
            items,
            seen,
        );
    }
}

#[derive(Debug)]
struct ImportPath {
    parts: Vec<String>,
}

fn parse_imports(source: &str) -> Vec<ImportPath> {
    let mut imports = Vec::new();

    for raw_line in source.lines() {
        let line = raw_line.trim();

        let Some(rest) = line.strip_prefix("import ") else {
            continue;
        };

        let path = rest.trim().trim_end_matches(';');

        if path.is_empty() {
            continue;
        }

        let parts = path
            .split('.')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();

        if parts.is_empty() {
            continue;
        }

        imports.push(ImportPath { parts });
    }

    imports
}

fn completion_kind(kind: SymbolKind) -> u32 {
    match kind {
        SymbolKind::Variable => 6,
        SymbolKind::Function => 3,
        SymbolKind::Class => 7,
        SymbolKind::Interface => 8,
        SymbolKind::Import => 9,
    }
}

fn completion_detail(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Variable => "Variable",
        SymbolKind::Function => "Function",
        SymbolKind::Class => "Class",
        SymbolKind::Interface => "Interface",
        SymbolKind::Import => "Import",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kastel::frontend::lexer::lexer::Lexer;
    use kastel::frontend::parser::Parser;

    fn build_index(source: &str) -> SymbolIndex {
        let mut lexer = Lexer::new(source.to_owned());

        let tokens = lexer.scan_token().expect("lexer failed");

        let mut parser = Parser::new(tokens);

        let statements = parser.parse().expect("parser failed");

        let mut index = SymbolIndex::new();

        index.rebuild(source, &statements);

        index
    }

    fn labels(items: &[Value]) -> Vec<String> {
        items
            .iter()
            .filter_map(|item| item["label"].as_str().map(str::to_string))
            .collect()
    }

    fn sort_text(item: &Value) -> &str {
        item["sortText"].as_str().expect("missing sortText")
    }

    // -----------------------------------------------------------
    // detect_member_access
    // -----------------------------------------------------------

    #[test]
    fn detect_member_access_simple() {
        let ctx = detect_member_access("math.", 5).expect("expected member context");

        assert_eq!(ctx.path, "math");
        assert!(!ctx.is_import_line);
    }

    #[test]
    fn detect_member_access_on_import_line() {
        let ctx = detect_member_access("import math.", 12).expect("expected member context");

        assert_eq!(ctx.path, "math");
        assert!(ctx.is_import_line);
    }

    #[test]
    fn detect_member_access_nested_path() {
        let ctx = detect_member_access("math.xx.", 8).expect("expected member context");

        assert_eq!(ctx.path, "math.xx");
    }

    #[test]
    fn detect_member_access_inside_expression() {
        let ctx = detect_member_access("println(math.", 13).expect("expected member context");

        assert_eq!(ctx.path, "math");
    }

    #[test]
    fn detect_member_access_returns_none_without_dot() {
        assert!(detect_member_access("math", 4,).is_none());
    }

    #[test]
    fn detect_member_access_returns_none_on_just_dot() {
        assert!(detect_member_access(".", 1,).is_none());
    }

    // -----------------------------------------------------------
    // keyword_completion / snippets (inchangés)
    // -----------------------------------------------------------

    #[test]
    fn parses_simple_import() {
        let imports = parse_imports("import math.xx\n");

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].parts, vec!["math", "xx"]);
    }

    #[test]
    fn parses_multiple_imports() {
        let imports = parse_imports(
            "import math.xx\n\
                 import utils.print\n",
        );

        assert_eq!(imports.len(), 2);
        assert_eq!(imports[1].parts, vec!["utils", "print"]);
    }

    #[test]
    fn completion_kind_mapping_is_stable() {
        assert_eq!(completion_kind(SymbolKind::Variable), 6);
        assert_eq!(completion_kind(SymbolKind::Function), 3);
        assert_eq!(completion_kind(SymbolKind::Class), 7);
    }

    #[test]
    fn local_completions_include_private_symbols() {
        let source = "const PRIVATE = 1\n\
             export const PUBLIC = 2\n";

        let symbols = build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(&symbols, "", Scope::Local, &mut items, &mut seen);

        let labels = labels(&items);

        assert!(labels.contains(&"PRIVATE".to_string()));
        assert!(labels.contains(&"PUBLIC".to_string()));
    }

    #[test]
    fn imported_completions_exclude_private_symbols() {
        let source = "const PRIVATE = 1\n\
             export const PUBLIC = 2\n";

        let symbols = build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(&symbols, "", Scope::Imported, &mut items, &mut seen);

        let labels = labels(&items);

        assert!(labels.contains(&"PUBLIC".to_string()));
        assert!(!labels.contains(&"PRIVATE".to_string()));
    }

    #[test]
    fn imported_completions_respect_prefix() {
        let source = "export const VALUE = 1\n\
             export const OTHER = 2\n";

        let symbols = build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(&symbols, "VA", Scope::Imported, &mut items, &mut seen);

        assert_eq!(labels(&items), vec!["VALUE".to_string()]);
    }

    #[test]
    fn local_symbols_sort_before_imported() {
        let source = "const LOCAL = 1\n";
        let symbols = build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(&symbols, "", Scope::Local, &mut items, &mut seen);

        assert_eq!(sort_text(&items[0]), "0_LOCAL");
    }

    #[test]
    fn imported_symbols_sort_with_prefix_one() {
        let source = "export const REMOTE = 1\n";
        let symbols = build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(&symbols, "", Scope::Imported, &mut items, &mut seen);

        assert_eq!(sort_text(&items[0]), "1_REMOTE");
    }

    #[test]
    fn imported_symbol_detail_is_prefixed() {
        let source = "export func greet() {\n\
             }\n";

        let symbols = build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(&symbols, "", Scope::Imported, &mut items, &mut seen);

        assert_eq!(items[0]["detail"], "Imported Function");
    }

    #[test]
    fn local_symbol_detail_has_no_prefix() {
        let source = "func greet() {\n\
             }\n";

        let symbols = build_index(source);

        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_document_completions(&symbols, "", Scope::Local, &mut items, &mut seen);

        assert_eq!(items[0]["detail"], "Function");
    }

    #[test]
    fn keyword_func_is_a_snippet() {
        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_keyword_completions("func", &mut items, &mut seen);

        assert_eq!(items.len(), 1);

        let item = &items[0];

        assert_eq!(item["label"], "func");
        assert_eq!(item["kind"], 15);
        assert_eq!(item["insertTextFormat"], 2);
        assert_eq!(item["insertText"], "func ${1:name}(${2}) {\n\t${0}\n}");
        assert_eq!(item["sortText"], "2_func");
    }

    #[test]
    fn keyword_true_is_not_a_snippet() {
        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_keyword_completions("true", &mut items, &mut seen);

        assert_eq!(items.len(), 1);

        let item = &items[0];

        assert_eq!(item["label"], "true");
        assert_eq!(item["kind"], 14);
        assert!(item.get("insertTextFormat").is_none());
    }

    #[test]
    fn keyword_completion_respects_prefix() {
        let mut items = Vec::new();
        let mut seen = HashSet::new();

        add_keyword_completions("wh", &mut items, &mut seen);

        assert_eq!(labels(&items), vec!["while".to_string()]);
    }
}
