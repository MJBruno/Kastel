//! Entrées/sorties fichier natives.
//!
//! Aucune de ces opérations n'est faisable en Kastel pur : lire/
//! écrire un fichier nécessite un appel système, que seul le runtime
//! (ici, `std::fs`) peut faire.

use std::collections::HashMap;
use std::fs;

use crate::{
    error::runtime_error::RuntimeError, runtime::value::Value,
};

fn expect_string(value: &Value) -> Result<String, RuntimeError> {
    value.as_string_value().ok_or(RuntimeError::TypeError)
}

fn io_error(operation: &str, path: &str, error: std::io::Error) -> RuntimeError {
    RuntimeError::ModuleError(format!("file.{operation}: {path}: {error}"))
}

pub fn native_file_read(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;

    let content = fs::read_to_string(&path).map_err(|error| io_error("read", &path, error))?;

    Ok(Value::new_string(content))
}

/// Lit le fichier et le découpe en lignes (`\n`/`\r\n` retirés), sans
/// ligne finale vide superflue si le fichier se termine par un saut
/// de ligne — comme `str.split("\n")` mais sans le piège du dernier
/// élément vide.
pub fn native_file_read_lines(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;

    let content =
        fs::read_to_string(&path).map_err(|error| io_error("read_lines", &path, error))?;

    let lines = content
        .lines()
        .map(|line| Value::new_string(line.to_string()))
        .collect::<Vec<_>>();

    Ok(Value::new_array(lines))
}

pub fn native_file_write(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;
    let content = expect_string(&args[1])?;

    fs::write(&path, content).map_err(|error| io_error("write", &path, error))?;

    Ok(Value::None)
}

pub fn native_file_append(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;
    let content = expect_string(&args[1])?;

    use std::io::Write as _;

    let mut handle = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| io_error("append", &path, error))?;

    handle
        .write_all(content.as_bytes())
        .map_err(|error| io_error("append", &path, error))?;

    Ok(Value::None)
}

pub fn native_file_exists(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;

    Ok(Value::Boolean(std::path::Path::new(&path).exists()))
}

pub fn native_file_delete(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;

    fs::remove_file(&path).map_err(|error| io_error("delete", &path, error))?;

    Ok(Value::None)
}

pub fn native_file_size(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;

    let metadata = fs::metadata(&path).map_err(|error| io_error("size", &path, error))?;

    let size = i64::try_from(metadata.len()).map_err(|_| {
        RuntimeError::ModuleError(format!(
            "file.size: {path}: file size exceeds Kastel integer range"
        ))
    })?;

    Ok(Value::Integer(size))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file_path() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after UNIX_EPOCH")
            .as_nanos();

        std::env::temp_dir().join(format!(
            "kastel_stdlib_file_test_{}_{}.txt",
            std::process::id(),
            stamp
        ))
    }

    fn string(value: &str) -> Value {
        Value::new_string(value.to_string())
    }

    #[test]
    fn file_lifecycle_covers_read_write_append_lines_exists_size_and_delete() {
        let path = temp_file_path();
        let path_value = string(path.to_string_lossy().as_ref());

        assert!(!matches!(
            native_file_exists(&[path_value.clone()]).unwrap(),
            Value::Boolean(true)
        ));

        native_file_write(&[path_value.clone(), string("a\nb")]).unwrap();
        assert_eq!(
            native_file_read(&[path_value.clone()]).unwrap(),
            string("a\nb")
        );

        let lines = native_file_read_lines(&[path_value.clone()]).unwrap();
        let Value::Object(handle) = lines else {
            panic!("file_read_lines() must return an array")
        };
        let crate::runtime::object::Object::Array(items) = &*handle.borrow() else {
            panic!("file_read_lines() must return an array")
        };
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], string("a"));
        assert_eq!(items[1], string("b"));

        assert!(matches!(
            native_file_exists(&[path_value.clone()]).unwrap(),
            Value::Boolean(true)
        ));
        assert_eq!(native_file_size(&[path_value.clone()]).unwrap(), Value::Integer(3));

        native_file_append(&[path_value.clone(), string("\nc")]).unwrap();
        assert_eq!(
            native_file_read(&[path_value.clone()]).unwrap(),
            string("a\nb\nc")
        );
        assert_eq!(native_file_size(&[path_value.clone()]).unwrap(), Value::Integer(5));

        native_file_delete(&[path_value.clone()]).unwrap();
        assert!(!matches!(
            native_file_exists(&[path_value.clone()]).unwrap(),
            Value::Boolean(true)
        ));
    }

    #[test]
    fn file_operations_enforce_argument_types_and_arities() {
        assert!(matches!(
            native_file_read(&[]),
            Err(RuntimeError::WrongArgumentCount { expected: 1, found: 0 })
        ));
        assert!(matches!(
            native_file_write(&[Value::Integer(1), string("x")]),
            Err(RuntimeError::TypeError)
        ));
        assert!(matches!(
            native_file_append(&[string("x")]),
            Err(RuntimeError::WrongArgumentCount { expected: 2, found: 1 })
        ));
    }
}

// ====================================================================
// ENREGISTREMENT
// ====================================================================

fn register_one(globals: &mut HashMap<String, Value>, name: &str, function: super::NativeFn) {
    globals.insert(name.to_string(), Value::NativeFunction(function));
}

pub fn register(globals: &mut HashMap<String, Value>) {
    register_one(globals, "file_read", native_file_read);
    register_one(globals, "file_read_lines", native_file_read_lines);
    register_one(globals, "file_write", native_file_write);
    register_one(globals, "file_append", native_file_append);
    register_one(globals, "file_exists", native_file_exists);
    register_one(globals, "file_delete", native_file_delete);
    register_one(globals, "file_size", native_file_size);
}

