//! Entrées/sorties fichier natives.
//!
//! Aucune de ces opérations n'est faisable en Kastel pur : lire/
//! écrire un fichier nécessite un appel système, que seul le runtime
//! (ici, `std::fs`) peut faire.

use std::collections::HashMap;
use std::fs;

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value,
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

    let content = fs::read_to_string(&path).map_err(|error| io_error("read_lines", &path, error))?;

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

    Ok(Value::Integer(metadata.len() as i64))
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
    register_one(globals, "file_read", native_file_read);
    register_one(globals, "file_read_lines", native_file_read_lines);
    register_one(globals, "file_write", native_file_write);
    register_one(globals, "file_append", native_file_append);
    register_one(globals, "file_exists", native_file_exists);
    register_one(globals, "file_delete", native_file_delete);
    register_one(globals, "file_size", native_file_size);
}

pub fn register_compiler(compiler: &mut Compiler) {
    define_one(compiler, "file_read");
    define_one(compiler, "file_read_lines");
    define_one(compiler, "file_write");
    define_one(compiler, "file_append");
    define_one(compiler, "file_exists");
    define_one(compiler, "file_delete");
    define_one(compiler, "file_size");
}
