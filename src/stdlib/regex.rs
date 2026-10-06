//! Expressions régulières natives de `std.regex`.
//!
//! La crate `regex` fournit un moteur Rust stable et non rétrogradable en
//! backtracking arbitraire. La surface Kastel reste petite et déterministe :
//! compilation du motif à chaque appel, résultats en `Result` et indices de
//! correspondance exprimés en caractères Unicode Kastel (pas en octets).
#[allow(unused_imports)]
use crate::{error::runtime_error::RuntimeError, runtime::object::Object, runtime::value::Value};
use std::collections::HashMap;

const MAX_PATTERN_BYTES: usize = 1024 * 1024;
const MAX_TEXT_BYTES: usize = 64 * 1024 * 1024;
const MAX_MATCHES: usize = 1_000_000;

fn module_error(message: impl Into<String>) -> RuntimeError {
    RuntimeError::ModuleError(format!("regex: {}", message.into()))
}

fn expect_string(value: &Value) -> Result<String, RuntimeError> {
    value.as_string_value().ok_or(RuntimeError::TypeError)
}

fn expect_args(args: &[Value], expected: usize) -> Result<(), RuntimeError> {
    if args.len() != expected {
        return Err(RuntimeError::WrongArgumentCount { expected, found: args.len() });
    }
    Ok(())
}

fn compile(pattern: &str) -> Result<regex::Regex, RuntimeError> {
    if pattern.len() > MAX_PATTERN_BYTES {
        return Err(module_error("pattern is too large"));
    }
    regex::Regex::new(pattern).map_err(|error| module_error(format!("invalid pattern: {error}")))
}

fn validate_text(text: &str) -> Result<(), RuntimeError> {
    if text.len() > MAX_TEXT_BYTES {
        return Err(module_error("input text is too large"));
    }
    Ok(())
}

fn char_index(text: &str, byte_index: usize) -> i64 {
    text[..byte_index].chars().count() as i64
}

fn capture_list(captures: &regex::Captures<'_>) -> Value {
    let mut groups = Vec::new();
    for index in 1..captures.len() {
        match captures.get(index) {
            Some(value) => groups.push(Value::new_some(Value::new_string(value.as_str().to_string()))),
            None => groups.push(Value::None),
        }
    }
    Value::new_array(groups)
}

fn match_record(text: &str, captures: &regex::Captures<'_>) -> Value {
    let full = captures
        .get(0)
        .expect("regex capture group 0 must exist");
    Value::new_record(vec![
        ("start".into(), Value::Integer(char_index(text, full.start()))),
        ("end".into(), Value::Integer(char_index(text, full.end()))),
        ("text".into(), Value::new_string(full.as_str().to_string())),
        ("groups".into(), capture_list(captures)),
    ])
}

pub fn native_regex_is_match(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;
    let pattern = expect_string(&args[0])?;
    let text = expect_string(&args[1])?;
    validate_text(&text)?;
    let regex = compile(&pattern)?;
    Ok(Value::new_ok(Value::Boolean(regex.is_match(&text))))
}

pub fn native_regex_find(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;
    let pattern = expect_string(&args[0])?;
    let text = expect_string(&args[1])?;
    validate_text(&text)?;
    let regex = compile(&pattern)?;
    let result = regex.captures(&text)
        .map(|captures| match_record(&text, &captures));
    Ok(Value::new_ok(Value::new_option(result)))
}

pub fn native_regex_find_all(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;
    let pattern = expect_string(&args[0])?;
    let text = expect_string(&args[1])?;
    validate_text(&text)?;
    let regex = compile(&pattern)?;
    let mut matches = Vec::new();
    for captures in regex.captures_iter(&text).take(MAX_MATCHES + 1) {
        if matches.len() == MAX_MATCHES {
            return Err(module_error("too many matches"));
        }
        matches.push(match_record(&text, &captures));
    }
    Ok(Value::new_ok(Value::new_array(matches)))
}

pub fn native_regex_replace(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 3)?;
    let pattern = expect_string(&args[0])?;
    let text = expect_string(&args[1])?;
    let replacement = expect_string(&args[2])?;
    validate_text(&text)?;
    if replacement.len() > MAX_TEXT_BYTES {
        return Err(module_error("replacement is too large"));
    }
    let regex = compile(&pattern)?;
    let result = regex.replace_all(&text, replacement.as_str()).into_owned();
    if result.len() > MAX_TEXT_BYTES {
        return Err(module_error("replacement result is too large"));
    }
    Ok(Value::new_ok(Value::new_string(result)))
}

pub fn native_regex_split(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;
    let pattern = expect_string(&args[0])?;
    let text = expect_string(&args[1])?;
    validate_text(&text)?;
    let regex = compile(&pattern)?;
    let pieces = regex.split(&text).take(MAX_MATCHES + 1).map(|piece| piece.to_string()).collect::<Vec<_>>();
    if pieces.len() > MAX_MATCHES {
        return Err(module_error("too many split pieces"));
    }
    Ok(Value::new_ok(Value::new_array(
        pieces.into_iter().map(Value::new_string).collect(),
    )))
}

pub fn native_regex_escape(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 1)?;
    let text = expect_string(&args[0])?;
    Ok(Value::new_string(regex::escape(&text)))
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("regex_is_match".into(), Value::NativeFunction(native_regex_is_match));
    globals.insert("regex_find".into(), Value::NativeFunction(native_regex_find));
    globals.insert("regex_find_all".into(), Value::NativeFunction(native_regex_find_all));
    globals.insert("regex_replace".into(), Value::NativeFunction(native_regex_replace));
    globals.insert("regex_split".into(), Value::NativeFunction(native_regex_split));
    globals.insert("regex_escape".into(), Value::NativeFunction(native_regex_escape));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(value: &str) -> Value { Value::new_string(value.to_string()) }

    #[test]
    fn regex_matches_and_reports_unicode_character_offsets() {
        let result = native_regex_find(&[s("é."), s("Aéx")]).unwrap();
        let Value::Object(handle) = result else { panic!("Result attendu"); };
        let object = handle.borrow();
        let Object::Result { ok: true, value } = &*object else { panic!("Ok attendu"); };
        let Value::Object(option) = value else { panic!("Option attendue"); };
        let Object::Option(Some(inner)) = &*option.borrow() else { panic!("Some attendu"); };
        let Value::Object(record) = inner else { panic!("record attendu"); };
        let Object::Record(fields) = &*record.borrow() else { panic!("record attendu"); };
        let start = fields.iter().find(|(name, _)| name == "start").unwrap().1.clone();
        let end = fields.iter().find(|(name, _)| name == "end").unwrap().1.clone();
        assert_eq!(start, Value::Integer(1));
        assert_eq!(end, Value::Integer(3));
    }

    #[test]
    fn regex_rejects_invalid_pattern() {
        let result = native_regex_is_match(&[s("["), s("abc")]).unwrap();
        let Value::Object(handle) = result else { panic!("Result attendu"); };
        assert!(matches!(&*handle.borrow(), Object::Result { ok: false, .. }));
    }

    #[test]
    fn regex_replace_and_split_are_deterministic() {
        let replaced = native_regex_replace(&[s("[0-9]+"), s("a12b34"), s("X")]).unwrap();
        let Value::Object(handle) = replaced else { panic!(); };
        let Object::Result { ok: true, value } = &*handle.borrow() else { panic!(); };
        assert_eq!(value.as_string_value().unwrap(), "aXbX");

        let split = native_regex_split(&[s(",\\s*"), s("a, b,c")]).unwrap();
        let Value::Object(handle) = split else { panic!(); };
        let Object::Result { ok: true, value } = &*handle.borrow() else { panic!(); };
        assert_eq!(value.to_string(), "[a, b, c]");
    }

    #[test]
    fn regex_escape_is_literal() {
        let value = native_regex_escape(&[s("a+b")]).unwrap();
        assert_eq!(value.as_string_value().unwrap(), "a\\+b");
    }
}
