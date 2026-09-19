//! Encodage/décodage JSON natifs.
//!
//! Nécessaire en natif (pas exprimable proprement en Kastel pur) :
//! un vrai parseur récursif avec gestion des échappements `\uXXXX`,
//! des nombres flottants/exposants, et un formatage compact fiable.
//! Écrire ça à la main en Kastel (via string.split/char_at) serait
//! à la fois beaucoup plus lent et beaucoup plus fragile.
//!
//! Mapping :
//!   JSON object -> dict (clés converties en chaînes)
//!   JSON array  -> array
//!   JSON string -> string
//!   JSON number -> integer (si pas de '.'/'e') sinon float
//!   true/false  -> boolean
//!   null        -> None
//!
//! À l'encodage, seuls integer/float/boolean/None/string/array/
//! tuple/dict (à clés string) sont supportés — une classe, une
//! closure, un module, etc. n'ont pas de représentation JSON
//! canonique et déclenchent une erreur claire plutôt qu'un résultat
//! silencieusement incorrect.

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError,
    runtime::gc_handle::Gc, runtime::object::Object, runtime::value::Value,
};

// ====================================================================
// ENCODE
// ====================================================================

pub fn native_json_encode(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let mut out = String::new();

    encode_value(&args[0], &mut out)?;

    Ok(Value::new_string(out))
}

fn encode_value(value: &Value, out: &mut String) -> Result<(), RuntimeError> {
    match value {
        Value::Integer(n) => {
            let _ = write!(out, "{n}");
            Ok(())
        }

        Value::Float(f) => {
            if f.is_finite() {
                let _ = write!(out, "{f}");
            } else {
                // JSON n'a pas NaN/Infinity : null est le repli le
                // plus courant (même comportement que JSON.stringify
                // en JavaScript).
                out.push_str("null");
            }

            Ok(())
        }

        Value::Boolean(b) => {
            out.push_str(if *b { "true" } else { "false" });
            Ok(())
        }

        Value::None => {
            out.push_str("null");
            Ok(())
        }

        Value::Object(handle) => encode_object(handle, out),

        _ => Err(unsupported_encode_type()),
    }
}

fn encode_object(handle: &Gc<Object>, out: &mut String) -> Result<(), RuntimeError> {
    match &*handle.borrow() {
        Object::String(text) => {
            encode_string(text, out);
            Ok(())
        }

        Object::Array(items) => {
            out.push('[');

            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }

                encode_value(item, out)?;
            }

            out.push(']');

            Ok(())
        }

        Object::Tuple(items) => {
            out.push('[');

            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }

                encode_value(item, out)?;
            }

            out.push(']');

            Ok(())
        }

        Object::Dict(pairs) => {
            out.push('{');

            for (index, (key, value)) in pairs.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }

                let key_string = key.as_string_value().ok_or_else(|| {
                    RuntimeError::ModuleError(
                        "json.encode: les clés de dict doivent être des chaînes".to_string(),
                    )
                })?;

                encode_string(&key_string, out);
                out.push(':');

                encode_value(value, out)?;
            }

            out.push('}');

            Ok(())
        }

        _ => Err(unsupported_encode_type()),
    }
}

fn encode_string(value: &str, out: &mut String) {
    out.push('"');

    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }

    out.push('"');
}

fn unsupported_encode_type() -> RuntimeError {
    RuntimeError::ModuleError(
        "json.encode: type non encodable en JSON (attendu : integer, float, boolean, None, \
             string, array, tuple, ou dict à clés string)"
            .to_string(),
    )
}

// ====================================================================
// DECODE
// ====================================================================

pub fn native_json_decode(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let text = args[0].as_string_value().ok_or(RuntimeError::TypeError)?;

    let mut parser = JsonParser {
        chars: text.chars().collect(),
        pos: 0,
    };

    let value = parser.parse_value()?;

    parser.skip_whitespace();

    if parser.pos != parser.chars.len() {
        return Err(parser.error("caractères en trop après la valeur JSON"));
    }

    Ok(value)
}

struct JsonParser {
    chars: Vec<char>,
    pos: usize,
}

impl JsonParser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let current = self.peek();

        if current.is_some() {
            self.pos += 1;
        }

        current
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.pos += 1;
        }
    }

    fn error(&self, message: &str) -> RuntimeError {
        RuntimeError::ModuleError(format!("json.decode: {message} (position {})", self.pos))
    }

    fn expect(&mut self, expected: char) -> Result<(), RuntimeError> {
        if self.advance() == Some(expected) {
            Ok(())
        } else {
            Err(self.error(&format!("'{expected}' attendu")))
        }
    }

    fn parse_value(&mut self) -> Result<Value, RuntimeError> {
        self.skip_whitespace();

        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => Ok(Value::new_string(self.parse_string()?)),
            Some('t') | Some('f') => self.parse_bool(),
            Some('n') => self.parse_null(),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            _ => Err(self.error("valeur JSON attendue")),
        }
    }

    fn parse_object(&mut self) -> Result<Value, RuntimeError> {
        self.expect('{')?;
        self.skip_whitespace();

        let mut pairs = Vec::new();

        if self.peek() == Some('}') {
            self.advance();
            return Ok(Value::new_dict(pairs));
        }

        loop {
            self.skip_whitespace();

            let key = self.parse_string()?;

            self.skip_whitespace();
            self.expect(':')?;

            let value = self.parse_value()?;

            pairs.push((Value::new_string(key), value));

            self.skip_whitespace();

            match self.advance() {
                Some(',') => continue,
                Some('}') => break,
                _ => return Err(self.error("',' ou '}' attendu")),
            }
        }

        Ok(Value::new_dict(pairs))
    }

    fn parse_array(&mut self) -> Result<Value, RuntimeError> {
        self.expect('[')?;
        self.skip_whitespace();

        let mut items = Vec::new();

        if self.peek() == Some(']') {
            self.advance();
            return Ok(Value::new_array(items));
        }

        loop {
            let value = self.parse_value()?;

            items.push(value);

            self.skip_whitespace();

            match self.advance() {
                Some(',') => continue,
                Some(']') => break,
                _ => return Err(self.error("',' ou ']' attendu")),
            }
        }

        Ok(Value::new_array(items))
    }

    fn parse_string(&mut self) -> Result<String, RuntimeError> {
        self.expect('"')?;

        let mut result = String::new();

        loop {
            match self.advance() {
                None => return Err(self.error("chaîne non terminée")),
                Some('"') => break,

                Some('\\') => match self.advance() {
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('/') => result.push('/'),
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('r') => result.push('\r'),
                    Some('b') => result.push('\u{8}'),
                    Some('f') => result.push('\u{c}'),
                    Some('u') => {
                        let mut code: u32 = 0;

                        for _ in 0..4 {
                            let digit = self
                                .advance()
                                .and_then(|c| c.to_digit(16))
                                .ok_or_else(|| self.error("échappement \\u invalide"))?;

                            code = code * 16 + digit;
                        }

                        result.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                    }
                    _ => return Err(self.error("échappement invalide")),
                },

                Some(c) => result.push(c),
            }
        }

        Ok(result)
    }

    fn parse_bool(&mut self) -> Result<Value, RuntimeError> {
        if self.chars[self.pos..].starts_with(&['t', 'r', 'u', 'e']) {
            self.pos += 4;
            Ok(Value::Boolean(true))
        } else if self.chars[self.pos..].starts_with(&['f', 'a', 'l', 's', 'e']) {
            self.pos += 5;
            Ok(Value::Boolean(false))
        } else {
            Err(self.error("littéral invalide"))
        }
    }

    fn parse_null(&mut self) -> Result<Value, RuntimeError> {
        if self.chars[self.pos..].starts_with(&['n', 'u', 'l', 'l']) {
            self.pos += 4;
            Ok(Value::None)
        } else {
            Err(self.error("littéral invalide"))
        }
    }

    fn parse_number(&mut self) -> Result<Value, RuntimeError> {
        let start = self.pos;
        let mut is_float = false;

        if self.peek() == Some('-') {
            self.pos += 1;
        }

        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }

        if self.peek() == Some('.') {
            is_float = true;
            self.pos += 1;

            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }

        if matches!(self.peek(), Some('e' | 'E')) {
            is_float = true;
            self.pos += 1;

            if matches!(self.peek(), Some('+' | '-')) {
                self.pos += 1;
            }

            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }

        let text: String = self.chars[start..self.pos].iter().collect();

        if is_float {
            text.parse::<f64>()
                .map(Value::Float)
                .map_err(|_| self.error("nombre invalide"))
        } else {
            text.parse::<i64>()
                .map(Value::Integer)
                .or_else(|_| text.parse::<f64>().map(Value::Float))
                .map_err(|_| self.error("nombre invalide"))
        }
    }
}

// ====================================================================
// ENREGISTREMENT
// ====================================================================

fn register_one(globals: &mut HashMap<String, Value>, name: &str, function: super::NativeFn) {
    globals.insert(name.to_string(), Value::NativeFunction(function));
}

fn define_one(compiler: &mut Compiler, name: &str) {
    let _ = compiler.define_native(name);
}

pub fn register(globals: &mut HashMap<String, Value>) {
    register_one(globals, "json_encode", native_json_encode);
    register_one(globals, "json_decode", native_json_decode);
}

pub fn register_compiler(compiler: &mut Compiler) {
    define_one(compiler, "json_encode");
    define_one(compiler, "json_decode");
}
