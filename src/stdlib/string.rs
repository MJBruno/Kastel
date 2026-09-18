use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value,
};

// ============================================================
//                         HELPERS
// ============================================================

fn expect_string(value: &Value) -> Result<String, RuntimeError> {
    value.as_string_value().ok_or(RuntimeError::TypeError)
}

fn expect_integer(value: &Value) -> Result<i64, RuntimeError> {
    match value {
        Value::Integer(value) => Ok(*value),
        _ => Err(RuntimeError::TypeError),
    }
}

fn expect_index(value: &Value) -> Result<usize, RuntimeError> {
    let index = expect_integer(value)?;

    if index < 0 {
        return Err(RuntimeError::IndexOutOfBounds);
    }

    usize::try_from(index).map_err(|_| RuntimeError::IndexOutOfBounds)
}

fn expect_non_negative_count(value: &Value) -> Result<usize, RuntimeError> {
    let count = expect_integer(value)?;

    if count < 0 {
        return Err(RuntimeError::TypeError);
    }

    usize::try_from(count).map_err(|_| RuntimeError::TypeError)
}

// ============================================================
//                         FORMAT
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FormatSpec {
    fill: char,
    align: Option<char>,
    sign: Option<char>,
    alternate: bool,
    zero: bool,
    width: Option<usize>,
    grouping: Option<char>,
    precision: Option<usize>,
    kind: Option<char>,
}

impl Default for FormatSpec {
    fn default() -> Self {
        Self {
            fill: ' ',
            align: None,
            sign: None,
            alternate: false,
            zero: false,
            width: None,
            grouping: None,
            precision: None,
            kind: None,
        }
    }
}

impl FormatSpec {
    fn parse(spec: &str) -> Result<Self, RuntimeError> {
        let chars: Vec<char> = spec.chars().collect();
        let mut index = 0;
        let mut result = Self::default();

        // [fill][align]
        if chars.len() >= 2 && matches!(chars[1], '<' | '>' | '^' | '=') {
            result.fill = chars[0];
            result.align = Some(chars[1]);
            index = 2;
        } else if chars.first().is_some_and(|c| matches!(c, '<' | '>' | '^' | '=')) {
            result.align = Some(chars[0]);
            index = 1;
        }

        // [sign]
        if let Some(&character) = chars.get(index) {
            if matches!(character, '+' | '-' | ' ') {
                result.sign = Some(character);
                index += 1;
            }
        }

        // [#]
        if chars.get(index) == Some(&'#') {
            result.alternate = true;
            index += 1;
        }

        // [0]
        if chars.get(index) == Some(&'0') {
            result.zero = true;
            index += 1;
        }

        // [width]
        let width_start = index;
        while chars.get(index).is_some_and(|c| c.is_ascii_digit()) {
            index += 1;
        }

        if index > width_start {
            result.width = Some(parse_usize(&chars[width_start..index])?);
        }

        // [,] ou [_]
        if chars.get(index).is_some_and(|c| matches!(c, ',' | '_')) {
            result.grouping = Some(chars[index]);
            index += 1;
        }

        // [.precision]
        if chars.get(index) == Some(&'.') {
            index += 1;
            let precision_start = index;

            while chars.get(index).is_some_and(|c| c.is_ascii_digit()) {
                index += 1;
            }

            if index == precision_start {
                return Err(RuntimeError::FormatError(
                    "la précision doit contenir au moins un chiffre".to_string(),
                ));
            }

            result.precision = Some(parse_usize(&chars[precision_start..index])?);
        }

        // [type]
        if let Some(&character) = chars.get(index) {
            result.kind = Some(character);
            index += 1;
        }

        if index != chars.len() {
            return Err(RuntimeError::FormatError(format!(
                "spécificateur invalide '{}': caractère inattendu '{}'",
                spec,
                chars[index]
            )));
        }

        Ok(result)
    }

    fn is_numeric(&self) -> bool {
        matches!(
            self.kind,
            Some('b' | 'c' | 'd' | 'o' | 'x' | 'X' | 'n' | 'f' | 'F' | 'e' | 'E' | 'g' | 'G' | '%')
        ) || self.sign.is_some()
            || self.alternate
            || self.zero
            || self.grouping.is_some()
            || self.align == Some('=')
    }
}

fn parse_usize(chars: &[char]) -> Result<usize, RuntimeError> {
    chars
        .iter()
        .collect::<String>()
        .parse::<usize>()
        .map_err(|_| RuntimeError::FormatError("largeur ou précision trop grande".to_string()))
}


fn format_error(message: impl Into<String>) -> RuntimeError {
    RuntimeError::FormatError(message.into())
}

fn apply_width(text: String, spec: &FormatSpec, numeric: bool) -> Result<String, RuntimeError> {
    let Some(width) = spec.width else {
        return Ok(text);
    };

    let length = text.chars().count();
    if length >= width {
        return Ok(text);
    }

    let padding = width - length;
    let align = spec.align.unwrap_or(if numeric { '>' } else { '<' });
    let fill = if numeric && spec.zero {
        '0'
    } else {
        spec.fill
    };

    match align {
        '<' => Ok(format!("{}{}", text, repeat_char(fill, padding))),
        '>' => Ok(format!("{}{}", repeat_char(fill, padding), text)),
        '^' => {
            let left = padding / 2;
            let right = padding - left;
            Ok(format!(
                "{}{}{}",
                repeat_char(fill, left),
                text,
                repeat_char(fill, right)
            ))
        }
        '=' if numeric => Ok(apply_numeric_equal_padding(text, fill, padding)),
        '=' => Err(format_error("l'alignement '=' est réservé aux nombres")),
        _ => Err(format_error(format!("alignement invalide '{}'", align))),
    }
}

fn repeat_char(character: char, count: usize) -> String {
    std::iter::repeat(character).take(count).collect()
}

fn apply_numeric_equal_padding(text: String, fill: char, padding: usize) -> String {
    let mut split = 0;
    let chars: Vec<char> = text.chars().collect();

    if chars.first().is_some_and(|c| matches!(c, '+' | '-' | ' ')) {
        split = 1;
    }

    if chars.get(split) == Some(&'0') && chars.get(split + 1).is_some_and(|c| matches!(c, 'b' | 'o' | 'x' | 'X')) {
        split += 2;
    }

    let prefix: String = chars[..split].iter().collect();
    let rest: String = chars[split..].iter().collect();

    format!("{}{}{}", prefix, repeat_char(fill, padding), rest)
}

fn apply_sign(negative: bool, sign: Option<char>) -> String {
    if negative {
        "-".to_string()
    } else {
        match sign {
            Some('+') => "+".to_string(),
            Some(' ') => " ".to_string(),
            _ => String::new(),
        }
    }
}

fn group_digits(digits: &str, separator: char, group_size: usize) -> String {
    let group_size = group_size.max(1);
    let mut result = String::with_capacity(digits.len() + digits.len() / group_size);
    let total = digits.chars().count();

    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (total - index) % group_size == 0 {
            result.push(separator);
        }
        result.push(character);
    }

    result
}

fn format_integer(value: i64, spec: FormatSpec) -> Result<String, RuntimeError> {
    let kind = spec.kind.unwrap_or('d');

    if !matches!(kind, 'b' | 'c' | 'd' | 'o' | 'x' | 'X' | 'n') {
        return Err(format_error(format!(
            "le format '{}' n'est pas valide pour un entier",
            kind
        )));
    }

    if spec.precision.is_some() {
        return Err(format_error(
            "la précision décimale n'est pas supportée pour les entiers; utilisez la largeur, par exemple {:08d}",
        ));
    }

    if spec.alternate && matches!(kind, 'd' | 'n') {
        return Err(format_error(
            "l'option '#' est réservée aux formats binaires, octaux et hexadécimaux",
        ));
    }

    if spec.grouping == Some(',') && matches!(kind, 'b' | 'o' | 'x' | 'X' | 'c') {
        return Err(format_error(
            "le séparateur ',' est réservé au format décimal; utilisez '_' pour b/o/x/X",
        ));
    }

    if kind == 'c' {
        if value < 0 || value > char::MAX as i64 || (0xD800..=0xDFFF).contains(&value) {
            return Err(format_error(format!(
                "{} ne peut pas être converti en caractère",
                value
            )));
        }

        let text = (value as u32)
            .try_into()
            .ok()
            .and_then(char::from_u32)
            .map(|c| c.to_string())
            .ok_or_else(|| format_error("valeur Unicode invalide"))?;

        if spec.sign.is_some() || spec.alternate || spec.zero || spec.grouping.is_some() {
            return Err(format_error(
                "les options numériques b/o/x/#/+/0 ne sont pas valides avec le format c",
            ));
        }

        return apply_width(text, &spec, false);
    }

    let negative = value < 0;
    let magnitude = value.unsigned_abs();

    let (mut digits, prefix) = match kind {
        'b' => (format!("{magnitude:b}"), if spec.alternate { "0b" } else { "" }),
        'o' => (format!("{magnitude:o}"), if spec.alternate { "0o" } else { "" }),
        'x' => (format!("{magnitude:x}"), if spec.alternate { "0x" } else { "" }),
        'X' => (format!("{magnitude:X}"), if spec.alternate { "0X" } else { "" }),
        'd' | 'n' => (magnitude.to_string(), ""),
        _ => unreachable!(),
    };

    if let Some(separator) = spec.grouping {
        let group_size = if matches!(kind, 'b' | 'o' | 'x' | 'X') && separator == '_' {
            4
        } else {
            3
        };

        digits = group_digits(&digits, separator, group_size);
    }

    let sign = apply_sign(negative, spec.sign);
    let text = format!("{sign}{prefix}{digits}");

    if spec.align == Some('=') || (spec.zero && spec.align.is_none()) {
        return apply_width(text, &FormatSpec { align: Some('='), ..spec }, true);
    }

    apply_width(text, &spec, true)
}

fn format_float(value: f64, spec: FormatSpec) -> Result<String, RuntimeError> {
    let is_plain_display = spec.kind.is_none()
        && spec.precision.is_none()
        && spec.sign.is_none()
        && !spec.alternate
        && !spec.zero
        && spec.grouping.is_none();

    if is_plain_display {
        return apply_width(Value::Float(value).to_string(), &spec, true);
    }

    let kind = spec.kind.unwrap_or(if spec.precision.is_some() { 'f' } else { 'g' });

    if !matches!(kind, 'f' | 'F' | 'e' | 'E' | 'g' | 'G' | '%') {
        return Err(format_error(format!(
            "le format '{}' n'est pas valide pour un nombre décimal",
            kind
        )));
    }

    let negative = value.is_sign_negative();
    let mut absolute = value.abs();

    if kind == '%' {
        absolute *= 100.0;
    }

    let text_number = if absolute.is_nan() {
        if matches!(kind, 'F' | 'E' | 'G') {
            "NAN".to_string()
        } else {
            "nan".to_string()
        }
    } else if absolute.is_infinite() {
        if matches!(kind, 'F' | 'E' | 'G') {
            "INF".to_string()
        } else {
            "inf".to_string()
        }
    } else {
        match kind {
            'f' | 'F' | '%' => {
                let precision = spec.precision.unwrap_or(6);
                format!("{:.*}", precision, absolute)
            }
            'e' | 'E' => {
                let precision = spec.precision.unwrap_or(6);
                let mut rendered = format!("{:.*e}", precision, absolute);
                if kind == 'E' {
                    rendered.make_ascii_uppercase();
                }
                rendered
            }
            'g' | 'G' => format_float_general(absolute, spec.precision.unwrap_or(6), kind == 'G'),
            _ => unreachable!(),
        }
    };

    let mut rendered = text_number;

    if spec.alternate
        && matches!(kind, 'f' | 'F' | 'g' | 'G' | '%')
        && !rendered.contains('.')
        && !rendered.contains('N')
        && !rendered.contains('I')
    {
        rendered.push('.');
    }

    if let Some(separator) = spec.grouping {
        if matches!(kind, 'f' | 'F' | 'g' | 'G' | '%') {
            rendered = group_decimal(&rendered, separator);
        }
    }

    if kind == '%' {
        rendered.push('%');
    }

    let sign = apply_sign(negative && !absolute.is_nan(), spec.sign);
    let text = format!("{sign}{rendered}");

    if spec.align == Some('=') || (spec.zero && spec.align.is_none()) {
        return apply_width(text, &FormatSpec { align: Some('='), ..spec }, true);
    }

    apply_width(text, &spec, true)
}

fn format_float_general(value: f64, precision: usize, uppercase: bool) -> String {
    let precision = precision.max(1);

    if value == 0.0 {
        return "0".to_string();
    }

    let exponent = value.abs().log10().floor() as i32;

    let rendered = if exponent >= precision as i32 || exponent < -4 {
        let decimals = precision.saturating_sub(1);
        format!("{:.*e}", decimals, value)
    } else {
        let decimals = if exponent >= 0 {
            precision.saturating_sub(exponent as usize + 1)
        } else {
            precision + (-exponent - 1) as usize
        };

        format!("{:.*}", decimals, value)
    };

    let mut rendered = trim_float_zeros(rendered);

    if uppercase {
        rendered.make_ascii_uppercase();
    }

    rendered
}

fn trim_float_zeros(mut value: String) -> String {
    if let Some(exponent_index) = value.find(['e', 'E']) {
        let exponent = value.split_off(exponent_index);
        if let Some(dot) = value.find('.') {
            while value.ends_with('0') {
                value.pop();
            }
            if value.len() == dot + 1 {
                value.pop();
            }
        }
        value.push_str(&exponent);
        return value;
    }

    if let Some(dot) = value.find('.') {
        while value.ends_with('0') {
            value.pop();
        }
        if value.len() == dot + 1 {
            value.pop();
        }
    }

    value
}

fn group_decimal(value: &str, separator: char) -> String {
    let exponent_index = value.find(['e', 'E']);
    let (mantissa, exponent) = match exponent_index {
        Some(index) => (&value[..index], &value[index..]),
        None => (value, ""),
    };

    let (integer, fraction) = match mantissa.find('.') {
        Some(index) => (&mantissa[..index], &mantissa[index..]),
        None => (mantissa, ""),
    };

    let grouped = group_digits(integer, separator, 3);
    format!("{grouped}{fraction}{exponent}")
}

fn truncate_chars(value: &str, precision: usize) -> String {
    value.chars().take(precision).collect()
}

fn escape_string_repr(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');

    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other => escaped.push(other),
        }
    }

    escaped.push('"');
    escaped
}

fn value_repr(value: &Value) -> String {
    match value {
        Value::Object(handle) => match &*handle.borrow() {
            crate::runtime::object::Object::String(value) => escape_string_repr(value),
            _ => value.to_string(),
        },
        _ => value.to_string(),
    }
}

fn format_non_numeric(
    value: &Value,
    conversion: Option<char>,
    spec: FormatSpec,
) -> Result<String, RuntimeError> {
    if spec.is_numeric() {
        return Err(format_error(format!(
            "le format numérique '{}' n'est pas valide pour la valeur de type {}",
            spec.kind.map(|c| c.to_string()).unwrap_or_default(),
            value.type_name()
        )));
    }

    let mut text = match conversion {
        Some('r') => value_repr(value),
        Some('s') | None => value.to_string(),
        Some(other) => {
            return Err(format_error(format!(
                "conversion '!{}' inconnue; utilisez !s ou !r",
                other
            )))
        }
    };

    if let Some(kind) = spec.kind {
        match kind {
            's' => {}
            'r' => text = value_repr(value),
            _ => {
                return Err(format_error(format!(
                    "le format '{}' n'est pas valide pour {}",
                    kind,
                    value.type_name()
                )))
            }
        }
    }

    if let Some(precision) = spec.precision {
        text = truncate_chars(&text, precision);
    }

    apply_width(text, &spec, false)
}

fn format_value(
    value: &Value,
    conversion: Option<char>,
    spec: FormatSpec,
) -> Result<String, RuntimeError> {
    if spec == FormatSpec::default() {
        match conversion {
            None | Some('s') => return Ok(value.to_string()),
            Some('r') => return Ok(value_repr(value)),
            Some(other) => {
                return Err(format_error(format!(
                    "conversion !{} non valide pour {}",
                    other,
                    value.type_name()
                )))
            }
        }
    }

    match value {
        Value::Integer(number) => {
            if conversion.is_some_and(|conversion| !matches!(conversion, 'r' | 's')) {
                return Err(format_error(format!(
                    "conversion !{} non valide pour int",
                    conversion.unwrap_or(' ')
                )));
            }
            format_integer(*number, spec)
        }

        Value::Float(number) => {
            if conversion.is_some_and(|conversion| !matches!(conversion, 'r' | 's')) {
                return Err(format_error(format!(
                    "conversion !{} non valide pour float",
                    conversion.unwrap_or(' ')
                )));
            }
            format_float(*number, spec)
        }

        _ => format_non_numeric(value, conversion, spec),
    }
}

fn parse_placeholder(
    content: &str,
    automatic_index: &mut usize,
    used_automatic: &mut bool,
    used_explicit: &mut bool,
    args: &[Value],
) -> Result<String, RuntimeError> {
    let mut field = content;
    let mut conversion = None;
    let mut format_spec = "";

    if let Some(colon_index) = field.find(':') {
        format_spec = &field[colon_index + 1..];
        field = &field[..colon_index];
    }

    if let Some(exclamation_index) = field.find('!') {
        conversion = field[exclamation_index + 1..].chars().next();

        if field[exclamation_index + 1..].chars().count() != 1 {
            return Err(format_error("une conversion doit être !s ou !r"));
        }

        field = &field[..exclamation_index];
    }

    let argument_index = if field.is_empty() {
        *used_automatic = true;

        if *used_explicit {
            return Err(format_error(
                "impossible de mélanger des champs automatiques {} et indexés {0}",
            ));
        }

        let index = *automatic_index;
        *automatic_index += 1;
        index
    } else if field.chars().all(|character| character.is_ascii_digit()) {
        *used_explicit = true;

        if *used_automatic {
            return Err(format_error(
                "impossible de mélanger des champs automatiques {} et indexés {0}",
            ));
        }

        field
            .parse::<usize>()
            .map_err(|_| format_error("index d'argument trop grand"))?
    } else {
        return Err(format_error(format!(
            "nom de champ '{}' non supporté; utilisez {}, {0}, {1}, ...",
            field, "{}"
        )));
    };

    let value = args.get(argument_index).ok_or(RuntimeError::WrongArgumentCount {
        expected: argument_index + 1,
        found: args.len(),
    })?;

    let spec = FormatSpec::parse(format_spec)?;
    format_value(value, conversion, spec)
}

pub(crate) fn format_string(format: &str, args: &[Value]) -> Result<String, RuntimeError> {
    let chars: Vec<char> = format.chars().collect();
    let mut result = String::with_capacity(format.len());
    let mut index = 0;
    let mut automatic_index = 0;
    let mut used_automatic = false;
    let mut used_explicit = false;
    let mut used_arguments = 0usize;

    while index < chars.len() {
        match chars[index] {
            '{' => {
                if chars.get(index + 1) == Some(&'{') {
                    result.push('{');
                    index += 2;
                    continue;
                }

                let start = index + 1;
                let mut end = start;
                let mut nested = false;

                while end < chars.len() && chars[end] != '}' {
                    if chars[end] == '{' {
                        nested = true;
                        break;
                    }
                    end += 1;
                }

                if nested {
                    return Err(format_error("accolade '{' inattendue dans un champ"));
                }

                if end >= chars.len() {
                    return Err(format_error("accolade '{' non fermée"));
                }

                let content: String = chars[start..end].iter().collect();
                let rendered = parse_placeholder(
                    &content,
                    &mut automatic_index,
                    &mut used_automatic,
                    &mut used_explicit,
                    args,
                )?;

                result.push_str(&rendered);
                used_arguments = used_arguments.max(automatic_index);
                index = end + 1;
            }

            '}' => {
                if chars.get(index + 1) == Some(&'}') {
                    result.push('}');
                    index += 2;
                    continue;
                }

                return Err(format_error("accolade '}' sans ouverture correspondante"));
            }

            character => {
                result.push(character);
                index += 1;
            }
        }
    }

    if used_automatic && used_arguments < args.len() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: used_arguments,
            found: args.len(),
        });
    }

    Ok(result)
}

// ============================================================
//                         GLOBAL NATIVES
// ============================================================

pub fn native_format(args: &[Value]) -> Result<Value, RuntimeError> {
    let Some(first) = args.first() else {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: 0,
        });
    };

    let format = expect_string(first)?;

    Ok(Value::new_string(format_string(&format, &args[1..])?))
}

// pub fn native_strlen(args: &[Value]) -> Result<Value, RuntimeError> {
//     if args.len() != 1 {
//         return Err(RuntimeError::WrongArgumentCount {
//             expected: 1,
//             found: args.len(),
//         });
//     }

//     let value = expect_string(&args[0])?;

//     Ok(Value::Integer(value.chars().count() as i64))
// }

pub fn native_lower(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(expect_string(&args[0])?.to_lowercase()))
}

pub fn native_upper(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(expect_string(&args[0])?.to_uppercase()))
}

pub fn native_trim(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(
        expect_string(&args[0])?.trim().to_string(),
    ))
}

pub fn native_starts_with(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let prefix = expect_string(&args[1])?;

    Ok(Value::Boolean(value.starts_with(&prefix)))
}

pub fn native_ends_with(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let suffix = expect_string(&args[1])?;

    Ok(Value::Boolean(value.ends_with(&suffix)))
}

pub fn native_contains(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let search = expect_string(&args[1])?;

    Ok(Value::Boolean(value.contains(&search)))
}

pub fn native_replace(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let from = expect_string(&args[1])?;
    let to = expect_string(&args[2])?;

    Ok(Value::new_string(value.replacen(&from, &to, 1)))
}

pub fn native_split(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let separator = expect_string(&args[1])?;

    let elements = value
        .split(&separator)
        .map(|part| Value::new_string(part.to_string()))
        .collect();

    Ok(Value::new_array(elements))
}

// ============================================================
//                       STRING METHODS
// ============================================================

pub fn native_length(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    Ok(Value::Integer(value.chars().count() as i64))
}

pub fn native_get(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let index = expect_index(&args[1])?;

    value
        .chars()
        .nth(index)
        .map(|character| Value::new_string(character.to_string()))
        .ok_or(RuntimeError::IndexOutOfBounds)
}

pub fn native_contains_method(args: &[Value]) -> Result<Value, RuntimeError> {
    native_contains(args)
}

pub fn native_starts_with_method(args: &[Value]) -> Result<Value, RuntimeError> {
    native_starts_with(args)
}

pub fn native_ends_with_method(args: &[Value]) -> Result<Value, RuntimeError> {
    native_ends_with(args)
}

pub fn native_index_of(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let search = expect_string(&args[1])?;

    let index = value
        .find(&search)
        .map(|byte_index| value[..byte_index].chars().count() as i64)
        .unwrap_or(-1);

    Ok(Value::Integer(index))
}

pub fn native_last_index_of(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let search = expect_string(&args[1])?;

    let index = value
        .rfind(&search)
        .map(|byte_index| value[..byte_index].chars().count() as i64)
        .unwrap_or(-1);

    Ok(Value::Integer(index))
}

pub fn native_slice(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let start = expect_integer(&args[1])?;
    let end = expect_integer(&args[2])?;

    let length = value.chars().count() as i64;

    let normalize = |index: i64| -> usize {
        if index < 0 {
            (length + index).max(0) as usize
        } else {
            index.min(length) as usize
        }
    };

    let start = normalize(start);
    let end = normalize(end);

    if start >= end {
        return Ok(Value::new_string(String::new()));
    }

    let result: String = value.chars().skip(start).take(end - start).collect();

    Ok(Value::new_string(result))
}

pub fn native_substring(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let start = expect_integer(&args[1])?;
    let length = expect_integer(&args[2])?;

    if start < 0 || length < 0 {
        return Err(RuntimeError::TypeError);
    }

    let start = start as usize;
    let length = length as usize;

    let result: String = value.chars().skip(start).take(length).collect();

    Ok(Value::new_string(result))
}

pub fn native_trim_start(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(
        expect_string(&args[0])?.trim_start().to_string(),
    ))
}

pub fn native_trim_end(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(
        expect_string(&args[0])?.trim_end().to_string(),
    ))
}

pub fn native_replace_all(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let from = expect_string(&args[1])?;
    let to = expect_string(&args[2])?;

    Ok(Value::new_string(value.replace(&from, &to)))
}

pub fn native_join(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let separator = expect_string(&args[0])?;

    let values = match &args[1] {
        Value::Object(handle) => {
            let object = handle.borrow();

            match &*object {
                crate::runtime::object::Object::Array(values) => values.clone(),
                crate::runtime::object::Object::Tuple(values) => values.clone(),

                _ => return Err(RuntimeError::TypeError),
            }
        }

        _ => return Err(RuntimeError::TypeError),
    };

    let mut result = String::new();

    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            result.push_str(&separator);
        }

        result.push_str(&value.to_string());
    }

    Ok(Value::new_string(result))
}

pub fn native_repeat(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let count = expect_non_negative_count(&args[1])?;

    Ok(Value::new_string(value.repeat(count)))
}

pub fn native_char_at(args: &[Value]) -> Result<Value, RuntimeError> {
    native_get(args)
}

pub fn native_to_int(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    let parsed = value
        .trim()
        .parse::<i64>()
        .map_err(|_| RuntimeError::TypeError)?;

    Ok(Value::Integer(parsed))
}

pub fn native_to_float(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    let parsed = value
        .trim()
        .parse::<f64>()
        .map_err(|_| RuntimeError::TypeError)?;

    Ok(Value::Float(parsed))
}

pub fn native_is_empty(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    Ok(Value::Boolean(value.is_empty()))
}

pub fn native_is_digit(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    Ok(Value::Boolean(
        !value.is_empty() && value.chars().all(|c| c.is_ascii_digit()),
    ))
}

pub fn native_is_alpha(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    Ok(Value::Boolean(
        !value.is_empty() && value.chars().all(|c| c.is_alphabetic()),
    ))
}

pub fn native_is_alphanumeric(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    Ok(Value::Boolean(
        !value.is_empty() && value.chars().all(|c| c.is_alphanumeric()),
    ))
}

pub fn native_reverse(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    let result: String = value.chars().rev().collect();

    Ok(Value::new_string(result))
}

// ============================================================
//                     METHOD DISPATCH
// ============================================================

pub fn dispatch_method(name: &str, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    let result = match name {
        "length" => Some(native_length(args)?),

        "get" => Some(native_get(args)?),

        "contains" => Some(native_contains_method(args)?),

        "starts_with" => Some(native_starts_with_method(args)?),

        "ends_with" => Some(native_ends_with_method(args)?),

        "index_of" => Some(native_index_of(args)?),

        "last_index_of" => Some(native_last_index_of(args)?),

        "slice" => Some(native_slice(args)?),

        "substring" => Some(native_substring(args)?),

        "upper" => Some(native_upper(args)?),

        "lower" => Some(native_lower(args)?),

        "trim" => Some(native_trim(args)?),

        "trim_start" => Some(native_trim_start(args)?),

        "trim_end" => Some(native_trim_end(args)?),

        "replace" => Some(native_replace(args)?),

        "replace_all" => Some(native_replace_all(args)?),

        "split" => Some(native_split(args)?),

        "join" => Some(native_join(args)?),

        "repeat" => Some(native_repeat(args)?),

        "char_at" => Some(native_char_at(args)?),

        "to_int" => Some(native_to_int(args)?),

        "to_float" => Some(native_to_float(args)?),

        "is_empty" => Some(native_is_empty(args)?),

        "is_digit" => Some(native_is_digit(args)?),

        "is_alpha" => Some(native_is_alpha(args)?),

        "is_alphanumeric" => Some(native_is_alphanumeric(args)?),

        "reverse" => Some(native_reverse(args)?),

        _ => return Ok(None),
    };

    Ok(result)
}

// ============================================================
//                      GLOBAL REGISTRATION
// ============================================================

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("format".to_string(), Value::NativeFunction(native_format));
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("format");
}

#[cfg(test)]
mod tests {
    use super::format_string;
    use crate::runtime::value::Value;

    fn fmt(template: &str, args: Vec<Value>) -> String {
        format_string(template, &args).expect(template)
    }

    #[test]
    fn formats_integer_bases_sign_width_and_grouping() {
        assert_eq!(fmt("{}", vec![Value::Integer(42)]), "42");
        assert_eq!(fmt("{:+d}", vec![Value::Integer(42)]), "+42");
        assert_eq!(fmt("{:08d}", vec![Value::Integer(42)]), "00000042");
        assert_eq!(fmt("{:#b}", vec![Value::Integer(42)]), "0b101010");
        assert_eq!(fmt("{:#o}", vec![Value::Integer(42)]), "0o52");
        assert_eq!(fmt("{:#x}", vec![Value::Integer(42)]), "0x2a");
        assert_eq!(fmt("{:#X}", vec![Value::Integer(42)]), "0X2A");
        assert_eq!(fmt("{:,}", vec![Value::Integer(1234567)]), "1,234,567");
        assert_eq!(fmt("{:_}", vec![Value::Integer(1234567)]), "1_234_567");
        assert_eq!(fmt("{:>8d}", vec![Value::Integer(42)]), "      42");
        assert_eq!(fmt("{:^8d}", vec![Value::Integer(42)]), "   42   ");
    }

    #[test]
    fn formats_floats_precision_modes_and_percent() {
        let value = Value::Float(1.23456789);

        assert_eq!(fmt("{}", vec![value.clone()]), "1.23456789");
        assert_eq!(fmt("{:.4}", vec![value.clone()]), "1.2346");
        assert_eq!(fmt("{:.2f}", vec![value.clone()]), "1.23");
        assert_eq!(fmt("{:.2e}", vec![value.clone()]), "1.23e0");
        assert_eq!(fmt("{:.2E}", vec![value.clone()]), "1.23E0");
        assert_eq!(fmt("{:.2%}", vec![value.clone()]), "123.46%");
        assert_eq!(fmt("{:+08.2f}", vec![value]), "+0001.23");
    }

    #[test]
    fn formats_string_conversion_width_and_precision() {
        let value = Value::new_string("Kastel".to_string());

        assert_eq!(fmt("{}", vec![value.clone()]), "Kastel");
        assert_eq!(fmt("{!r}", vec![value.clone()]), "\"Kastel\"");
        assert_eq!(fmt("{:10}", vec![value.clone()]), "Kastel    ");
        assert_eq!(fmt("{:>10}", vec![value.clone()]), "    Kastel");
        assert_eq!(fmt("{:^10}", vec![value.clone()]), "  Kastel  ");
        assert_eq!(fmt("{:.3s}", vec![value]), "Kas");
    }

    #[test]
    fn formats_containers_with_display_alignment() {
        let array = Value::new_array(vec![Value::Integer(1), Value::Integer(2)]);
        let tuple = Value::new_tuple(vec![Value::Integer(1), Value::Integer(2)]);
        let dict = Value::new_dict(vec![
            (Value::new_string("name".to_string()), Value::new_string("Bruno".to_string())),
            (Value::new_string("age".to_string()), Value::Integer(20)),
        ]);

        assert_eq!(fmt("{}", vec![array.clone()]), "[1, 2]");
        assert_eq!(fmt("{}", vec![tuple.clone()]), "(1, 2)");
        assert_eq!(fmt("{}", vec![dict.clone()]), "{\"name\": \"Bruno\", \"age\": 20}");
        assert_eq!(fmt("{:>10}", vec![array]), "     [1, 2]");
        assert_eq!(fmt("{:^10}", vec![tuple]), "  (1, 2)  ");
        assert_eq!(fmt("{:.8}", vec![dict]), "{\"name\": \"Bruno\"}");
    }

    #[test]
    fn supports_escaped_braces_and_explicit_indexes() {
        assert_eq!(fmt("{{ {} }}", vec![Value::Integer(42)]), "{ 42 }");
        assert_eq!(fmt("{1} {0}", vec![Value::new_string("A".to_string()), Value::Integer(7)]), "7 A");
    }
}
