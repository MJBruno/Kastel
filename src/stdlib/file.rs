//! Entrées/sorties fichier natives.
//!
//! Opérations stateless de `std.fs` et handles persistants `File`/`OpenOptions`
//! pour `std.io`. Les handles sont intégrés à l’objet runtime et au GC.

use std::collections::HashMap;
use std::fs;
use std::io::{Read as IoRead, Seek as IoSeek, SeekFrom as IoSeekFrom, Write as IoWrite};
use std::path::Path;

use crate::{
    error::runtime_error::RuntimeError, runtime::object::Object, runtime::value::Value,
};

fn expect_string(value: &Value) -> Result<String, RuntimeError> {
    value.as_string_value().ok_or(RuntimeError::TypeError)
}

fn io_error(operation: &str, path: &str, error: std::io::Error) -> RuntimeError {
    RuntimeError::ModuleError(format!("file.{operation}: {path}: {error}"))
}

fn expect_byte_list(value: &Value) -> Result<Vec<u8>, RuntimeError> {
    let Value::Object(handle) = value else {
        return Err(RuntimeError::TypeError);
    };

    let object = handle.borrow();
    let (Object::Array(elements) | Object::Tuple(elements)) = &*object else {
        return Err(RuntimeError::TypeError);
    };

    elements
        .iter()
        .map(|value| match value {
            Value::Integer(byte) if (0..=255).contains(byte) => Ok(*byte as u8),
            _ => Err(RuntimeError::TypeError),
        })
        .collect()
}

fn bytes_value(bytes: Vec<u8>) -> Value {
    Value::new_array(
        bytes
            .into_iter()
            .map(|byte| Value::Integer(i64::from(byte)))
            .collect(),
    )
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

pub fn native_file_read_bytes(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;
    let bytes = fs::read(&path).map_err(|error| io_error("read_bytes", &path, error))?;

    Ok(bytes_value(bytes))
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

pub fn native_file_write_bytes(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;
    let bytes = expect_byte_list(&args[1])?;

    fs::write(&path, bytes).map_err(|error| io_error("write_bytes", &path, error))?;

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

    Ok(Value::Boolean(Path::new(&path).exists()))
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

pub fn native_file_copy(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let source = expect_string(&args[0])?;
    let destination = expect_string(&args[1])?;

    fs::copy(&source, &destination)
        .map_err(|error| io_error("copy", &format!("{source} -> {destination}"), error))?;

    Ok(Value::None)
}

pub fn native_file_rename(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let source = expect_string(&args[0])?;
    let destination = expect_string(&args[1])?;

    fs::rename(&source, &destination)
        .map_err(|error| io_error("rename", &format!("{source} -> {destination}"), error))?;

    Ok(Value::None)
}

pub fn native_file_canonicalize(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;
    let canonical =
        fs::canonicalize(&path).map_err(|error| io_error("canonicalize", &path, error))?;

    Ok(Value::new_string(canonical.to_string_lossy().into_owned()))
}

pub fn native_file_metadata(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;
    let metadata = fs::symlink_metadata(&path)
        .map_err(|error| io_error("metadata", &path, error))?;

    let size = i64::try_from(metadata.len()).map_err(|_| {
        RuntimeError::ModuleError(format!(
            "file.metadata: {path}: file size exceeds Kastel integer range"
        ))
    })?;

    Ok(Value::new_record(vec![
        ("size".to_string(), Value::Integer(size)),
        ("is_file".to_string(), Value::Boolean(metadata.is_file())),
        ("is_dir".to_string(), Value::Boolean(metadata.is_dir())),
        (
            "is_symlink".to_string(),
            Value::Boolean(metadata.file_type().is_symlink()),
        ),
        ("readonly".to_string(), Value::Boolean(metadata.permissions().readonly())),
    ]))
}

pub fn native_file_create_dir(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;
    fs::create_dir(&path).map_err(|error| io_error("create_dir", &path, error))?;

    Ok(Value::None)
}

pub fn native_file_create_dir_all(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;
    fs::create_dir_all(&path).map_err(|error| io_error("create_dir_all", &path, error))?;

    Ok(Value::None)
}

pub fn native_file_remove_dir(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;
    fs::remove_dir(&path).map_err(|error| io_error("remove_dir", &path, error))?;

    Ok(Value::None)
}

pub fn native_file_remove_dir_all(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;
    fs::remove_dir_all(&path).map_err(|error| io_error("remove_dir_all", &path, error))?;

    Ok(Value::None)
}

pub fn native_file_read_dir(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;
    let mut entries = fs::read_dir(&path)
        .map_err(|error| io_error("read_dir", &path, error))?
        .map(|entry| {
            let entry = entry.map_err(|error| io_error("read_dir", &path, error))?;
            Ok(Value::new_string(
                entry.file_name().to_string_lossy().into_owned(),
            ))
        })
        .collect::<Result<Vec<_>, RuntimeError>>()?;

    entries.sort_by(|left, right| left.to_string().cmp(&right.to_string()));

    Ok(Value::new_array(entries))
}

// ====================================================================
// HANDLES PERSISTANTS : File / OpenOptions
// ====================================================================

const MAX_READ_CHUNK: usize = 16 * 1024 * 1024;

fn file_handle(
    value: &Value,
) -> Result<std::rc::Rc<std::cell::RefCell<crate::runtime::file::FileState>>, RuntimeError> {
    let Value::Object(handle) = value else {
        return Err(RuntimeError::TypeError);
    };
    match &*handle.borrow() {
        Object::File(file) => Ok(file.clone()),
        _ => Err(RuntimeError::TypeError),
    }
}

fn open_options_handle(
    value: &Value,
) -> Result<std::rc::Rc<std::cell::RefCell<crate::runtime::file::OpenOptionsState>>, RuntimeError> {
    let Value::Object(handle) = value else {
        return Err(RuntimeError::TypeError);
    };
    match &*handle.borrow() {
        Object::OpenOptions(options) => Ok(options.clone()),
        _ => Err(RuntimeError::TypeError),
    }
}

fn method_arity(args: &[Value], expected: usize) -> Result<(), RuntimeError> {
    // args[0] est le receveur; `expected` ne compte que les arguments explicites.
    let found = args.len().saturating_sub(1);
    if args.is_empty() || found != expected {
        return Err(RuntimeError::WrongArgumentCount { expected, found });
    }
    Ok(())
}

fn handle_io_error(operation: &str, path: &Path, error: std::io::Error) -> RuntimeError {
    RuntimeError::ModuleError(format!("io.File.{operation}: {}: {error}", path.display()))
}

fn closed_file_error(operation: &str, path: &Path) -> RuntimeError {
    RuntimeError::ModuleError(format!(
        "io.File.{operation}: {}: file handle is closed",
        path.display()
    ))
}

pub fn native_io_file_open(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount { expected: 1, found: args.len() });
    }
    let path = expect_string(&args[0])?;
    let file = fs::File::open(&path).map_err(|error| io_error("open", &path, error))?;
    Ok(Value::new_file(crate::runtime::file::FileState::new(
        path.into(), file, true, false, false,
    )))
}

pub fn native_io_file_create(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount { expected: 1, found: args.len() });
    }
    let path = expect_string(&args[0])?;
    let file = fs::File::create(&path).map_err(|error| io_error("create", &path, error))?;
    Ok(Value::new_file(crate::runtime::file::FileState::new(
        path.into(), file, false, true, false,
    )))
}

pub fn native_io_open_options(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount { expected: 0, found: args.len() });
    }
    Ok(Value::new_open_options(crate::runtime::file::OpenOptionsState::new()))
}

/// Dispatch des méthodes d'un handle File. `args[0]` est le receveur.
pub fn dispatch_file_method(method: &str, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    const METHODS: &[&str] = &[
        "read", "read_to_end", "read_to_string", "write", "write_all", "flush",
        "seek", "stream_position", "set_len", "sync_all", "sync_data", "metadata",
        "try_clone", "close", "is_closed", "path",
    ];
    if !METHODS.contains(&method) {
        return Ok(None);
    }
    if args.is_empty() {
        return Err(RuntimeError::TypeError);
    }

    let state = file_handle(&args[0])?;
    if method == "is_closed" {
        method_arity(args, 0)?;
        return Ok(Some(Value::Boolean(state.borrow().is_closed())));
    }
    if method == "path" {
        method_arity(args, 0)?;
        return Ok(Some(Value::new_string(state.borrow().path.to_string_lossy().into_owned())));
    }
    if method == "close" {
        method_arity(args, 0)?;
        state.borrow_mut().close();
        return Ok(Some(Value::None));
    }
    if method == "try_clone" {
        method_arity(args, 0)?;
        let clone = state.borrow().try_clone().map_err(|error| {
            handle_io_error("try_clone", &state.borrow().path, error)
        })?;
        return Ok(Some(Value::new_file(clone)));
    }

    match method {
        "read" => {
            method_arity(args, 1)?;
            let requested = match args[1] {
                Value::Integer(value) if value >= 0 => usize::try_from(value).map_err(|_| RuntimeError::TypeError)?,
                _ => return Err(RuntimeError::TypeError),
            };
            if requested > MAX_READ_CHUNK {
                return Err(RuntimeError::ModuleError(format!(
                    "io.File.read: size must not exceed {MAX_READ_CHUNK} bytes; use smaller reads"
                )));
            }
            let mut state = state.borrow_mut();
            if !state.readable {
                return Err(RuntimeError::ModuleError(format!(
                    "io.File.read: {}: file was not opened for reading", state.path.display()
                )));
            }
            let path = state.path.clone();
            let file = state.file.as_mut().ok_or_else(|| closed_file_error("read", &path))?;
            let mut buffer = vec![0u8; requested];
            let count = file.read(&mut buffer).map_err(|error| handle_io_error("read", &path, error))?;
            buffer.truncate(count);
            Ok(Some(bytes_value(buffer)))
        }
        "read_to_end" => {
            method_arity(args, 0)?;
            let mut state = state.borrow_mut();
            if !state.readable {
                return Err(RuntimeError::ModuleError(format!(
                    "io.File.read_to_end: {}: file was not opened for reading", state.path.display()
                )));
            }
            let path = state.path.clone();
            let file = state.file.as_mut().ok_or_else(|| closed_file_error("read_to_end", &path))?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).map_err(|error| handle_io_error("read_to_end", &path, error))?;
            Ok(Some(bytes_value(buffer)))
        }
        "read_to_string" => {
            method_arity(args, 0)?;
            let mut state = state.borrow_mut();
            if !state.readable {
                return Err(RuntimeError::ModuleError(format!(
                    "io.File.read_to_string: {}: file was not opened for reading", state.path.display()
                )));
            }
            let path = state.path.clone();
            let file = state.file.as_mut().ok_or_else(|| closed_file_error("read_to_string", &path))?;
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).map_err(|error| handle_io_error("read_to_string", &path, error))?;
            Ok(Some(Value::new_string(buffer)))
        }
        "write" | "write_all" => {
            method_arity(args, 1)?;
            let bytes = if let Some(text) = args[1].as_string_value() {
                text.into_bytes()
            } else {
                expect_byte_list(&args[1])?
            };
            let mut state = state.borrow_mut();
            if !state.writable {
                return Err(RuntimeError::ModuleError(format!(
                    "io.File.{method}: {}: file was not opened for writing", state.path.display()
                )));
            }
            let path = state.path.clone();
            let file = state.file.as_mut().ok_or_else(|| closed_file_error(method, &path))?;
            if method == "write" {
                let count = file.write(&bytes).map_err(|error| handle_io_error(method, &path, error))?;
                let count = i64::try_from(count).map_err(|_| RuntimeError::NativeError)?;
                Ok(Some(Value::Integer(count)))
            } else {
                file.write_all(&bytes).map_err(|error| handle_io_error(method, &path, error))?;
                Ok(Some(Value::None))
            }
        }
        "flush" | "sync_all" | "sync_data" => {
            method_arity(args, 0)?;
            let mut state = state.borrow_mut();
            let path = state.path.clone();
            let file = state.file.as_mut().ok_or_else(|| closed_file_error(method, &path))?;
            let result = match method {
                "flush" => file.flush(),
                "sync_all" => file.sync_all(),
                _ => file.sync_data(),
            };
            result.map_err(|error| handle_io_error(method, &path, error))?;
            Ok(Some(Value::None))
        }
        "seek" => {
            method_arity(args, 2)?;
            let offset = match args[1] {
                Value::Integer(value) => value,
                _ => return Err(RuntimeError::TypeError),
            };
            let origin = match &args[2] {
                Value::Object(handle) => match &*handle.borrow() {
                    Object::EnumVariant { enum_name, variant_name, .. }
                        if enum_name == "SeekFrom" => variant_name.clone(),
                    Object::String(value) => value.clone(),
                    _ => return Err(RuntimeError::TypeError),
                },
                _ => return Err(RuntimeError::TypeError),
            };
            let seek_from = match origin.as_str() {
                "Start" | "start" => IoSeekFrom::Start(u64::try_from(offset).map_err(|_| {
                    RuntimeError::ModuleError("io.File.seek: start offset must be non-negative".into())
                })?),
                "Current" | "current" => IoSeekFrom::Current(offset),
                "End" | "end" => IoSeekFrom::End(offset),
                _ => return Err(RuntimeError::ModuleError(
                    "io.File.seek: origin must be SeekFrom.Start, SeekFrom.Current or SeekFrom.End".into()
                )),
            };
            let mut state = state.borrow_mut();
            let path = state.path.clone();
            let file = state.file.as_mut().ok_or_else(|| closed_file_error("seek", &path))?;
            let position = file.seek(seek_from).map_err(|error| handle_io_error("seek", &path, error))?;
            let position = i64::try_from(position).map_err(|_| RuntimeError::ModuleError(
                "io.File.seek: position exceeds Kastel integer range".into()
            ))?;
            Ok(Some(Value::Integer(position)))
        }
        "stream_position" => {
            method_arity(args, 0)?;
            let mut state = state.borrow_mut();
            let path = state.path.clone();
            let file = state.file.as_mut().ok_or_else(|| closed_file_error("stream_position", &path))?;
            let position = file.stream_position().map_err(|error| handle_io_error("stream_position", &path, error))?;
            let position = i64::try_from(position).map_err(|_| RuntimeError::ModuleError(
                "io.File.stream_position: position exceeds Kastel integer range".into()
            ))?;
            Ok(Some(Value::Integer(position)))
        }
        "set_len" => {
            method_arity(args, 1)?;
            let length = match args[1] {
                Value::Integer(value) if value >= 0 => u64::try_from(value).map_err(|_| RuntimeError::TypeError)?,
                _ => return Err(RuntimeError::ModuleError("io.File.set_len: length must be non-negative".into())),
            };
            let state = state.borrow();
            let path = state.path.clone();
            let file = state.file.as_ref().ok_or_else(|| closed_file_error("set_len", &path))?;
            file.set_len(length).map_err(|error| handle_io_error("set_len", &path, error))?;
            Ok(Some(Value::None))
        }
        "metadata" => {
            method_arity(args, 0)?;
            let state = state.borrow();
            let path = state.path.clone();
            let file = state.file.as_ref().ok_or_else(|| closed_file_error("metadata", &path))?;
            let metadata = file.metadata().map_err(|error| handle_io_error("metadata", &path, error))?;
            let size = i64::try_from(metadata.len()).map_err(|_| RuntimeError::ModuleError(
                "io.File.metadata: file size exceeds Kastel integer range".into()
            ))?;
            Ok(Some(Value::new_record(vec![
                ("size".into(), Value::Integer(size)),
                ("is_file".into(), Value::Boolean(metadata.is_file())),
                ("is_dir".into(), Value::Boolean(metadata.is_dir())),
                ("readonly".into(), Value::Boolean(metadata.permissions().readonly())),
            ])))
        }
        _ => Ok(None),
    }
}

/// Dispatch des méthodes de configuration OpenOptions. `args[0]` est le receveur.
pub fn dispatch_open_options_method(method: &str, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    const FLAGS: &[&str] = &["read", "write", "append", "truncate", "create", "create_new"];
    if !FLAGS.contains(&method) && method != "open" {
        return Ok(None);
    }
    if args.is_empty() {
        return Err(RuntimeError::TypeError);
    }
    let state = open_options_handle(&args[0])?;
    if method == "open" {
        method_arity(args, 1)?;
        let path = expect_string(&args[1])?;
        let opened = state.borrow().open(Path::new(&path))
            .map_err(|error| io_error("open_options.open", &path, error))?;
        return Ok(Some(Value::new_file(opened)));
    }
    method_arity(args, 1)?;
    let enabled = match args[1] {
        Value::Boolean(value) => value,
        _ => return Err(RuntimeError::TypeError),
    };
    if !state.borrow_mut().set(method, enabled) {
        return Ok(None);
    }
    Ok(Some(args[0].clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file_path(suffix: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after UNIX_EPOCH")
            .as_nanos();

        std::env::temp_dir().join(format!(
            "kastel_stdlib_file_test_{}_{}_{}",
            std::process::id(),
            stamp,
            suffix
        ))
    }

    fn string(value: &str) -> Value {
        Value::new_string(value.to_string())
    }

    fn bytes(values: &[i64]) -> Value {
        Value::new_array(values.iter().copied().map(Value::Integer).collect())
    }

    #[test]
    fn file_lifecycle_covers_read_write_append_lines_exists_size_and_delete() {
        let path = temp_file_path("lifecycle.txt");
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
        let Object::Array(items) = &*handle.borrow() else {
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
    fn binary_file_operations_round_trip_bytes() {
        let path = temp_file_path("bytes.bin");
        let path_value = string(path.to_string_lossy().as_ref());
        let expected = bytes(&[0, 1, 2, 127, 255]);

        native_file_write_bytes(&[path_value.clone(), expected.clone()]).unwrap();
        assert_eq!(
            native_file_read_bytes(&[path_value.clone()]).unwrap(),
            expected
        );

        native_file_delete(&[path_value]).unwrap();
    }

    #[test]
    fn file_metadata_copy_rename_and_directory_operations_work() {
        let root = temp_file_path("tree");
        let nested = root.join("nested");
        let file = nested.join("source.txt");
        let copied = nested.join("copied.txt");
        let renamed = nested.join("renamed.txt");

        let root_value = string(root.to_string_lossy().as_ref());
        let nested_value = string(nested.to_string_lossy().as_ref());
        let file_value = string(file.to_string_lossy().as_ref());
        let copied_value = string(copied.to_string_lossy().as_ref());
        let renamed_value = string(renamed.to_string_lossy().as_ref());

        native_file_create_dir_all(&[nested_value.clone()]).unwrap();
        native_file_write(&[file_value.clone(), string("hello")]).unwrap();

        let metadata = native_file_metadata(&[file_value.clone()]).unwrap();
        let Value::Object(handle) = metadata else {
            panic!("metadata() must return a record")
        };
        let Object::Record(fields) = &*handle.borrow() else {
            panic!("metadata() must return a record")
        };
        assert!(fields.iter().any(|(name, value)| {
            name == "is_file" && value == &Value::Boolean(true)
        }));

        native_file_copy(&[file_value.clone(), copied_value.clone()]).unwrap();
        assert!(matches!(
            native_file_exists(&[copied_value.clone()]).unwrap(),
            Value::Boolean(true)
        ));

        native_file_rename(&[copied_value, renamed_value.clone()]).unwrap();
        assert!(matches!(
            native_file_exists(&[renamed_value]).unwrap(),
            Value::Boolean(true)
        ));

        let entries = native_file_read_dir(&[nested_value]).unwrap();
        let Value::Object(handle) = entries else {
            panic!("read_dir() must return an array")
        };
        let Object::Array(entries) = &*handle.borrow() else {
            panic!("read_dir() must return an array")
        };
        assert_eq!(entries.len(), 2);

        let canonical = native_file_canonicalize(&[file_value]).unwrap();
        let Value::Object(handle) = canonical else {
            panic!("canonicalize() must return a string")
        };
        let Object::String(canonical) = &*handle.borrow() else {
            panic!("canonicalize() must return a string")
        };
        assert!(Path::new(canonical).is_absolute());

        native_file_remove_dir_all(&[root_value]).unwrap();
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
            native_file_write_bytes(&[string("x"), bytes(&[256])]),
            Err(RuntimeError::TypeError)
        ));
        assert!(matches!(
            native_file_append(&[string("x")]),
            Err(RuntimeError::WrongArgumentCount { expected: 2, found: 1 })
        ));
    }
}

#[cfg(test)]
mod handle_tests {
    use super::*;
    use crate::runtime::object::Object;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn string(value: &str) -> Value {
        Value::new_string(value.to_string())
    }

    fn seek_origin(variant: &str) -> Value {
        Value::new_enum_variant("SeekFrom".into(), variant.into(), HashMap::new())
    }

    fn temporary_path() -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after UNIX_EPOCH")
            .as_nanos();
        std::env::temp_dir().join(format!("kastel_io_handle_{}_{}.bin", std::process::id(), stamp))
    }

    #[test]
    fn file_handle_supports_read_write_seek_clone_metadata_and_close() {
        let path = temporary_path();
        let path_text = path.to_string_lossy().into_owned();
        let options = native_io_open_options(&[]).unwrap();

        for (flag, enabled) in [
            ("read", true),
            ("write", true),
            ("create", true),
            ("truncate", true),
        ] {
            let updated = dispatch_open_options_method(
                flag,
                &[options.clone(), Value::Boolean(enabled)],
            )
            .unwrap()
            .expect("OpenOptions flag method should exist");
            assert_eq!(updated, options);
        }

        let file = dispatch_open_options_method("open", &[options, string(&path_text)])
            .unwrap()
            .expect("OpenOptions.open should exist");
        assert!(matches!(&file, Value::Object(handle) if matches!(&*handle.borrow(), Object::File(_))));

        assert_eq!(
            dispatch_file_method("write_all", &[file.clone(), string("hello")]).unwrap(),
            Some(Value::None)
        );
        assert_eq!(
            dispatch_file_method("seek", &[file.clone(), Value::Integer(0), seek_origin("Start")]).unwrap(),
            Some(Value::Integer(0))
        );
        assert_eq!(
            dispatch_file_method("read_to_string", &[file.clone()]).unwrap(),
            Some(string("hello"))
        );

        let cloned = dispatch_file_method("try_clone", &[file.clone()])
            .unwrap()
            .expect("File.try_clone should exist");
        assert_eq!(
            dispatch_file_method("is_closed", &[cloned.clone()]).unwrap(),
            Some(Value::Boolean(false))
        );
        let metadata = dispatch_file_method("metadata", &[file.clone()]).unwrap().unwrap();
        let Value::Object(record) = metadata else { panic!("metadata must be a record") };
        let record_ref = record.borrow();
        let Object::Record(fields) = &*record_ref else { panic!("metadata must be a record") };
        assert!(fields.iter().any(|(name, value)| name == "size" && *value == Value::Integer(5)));

        dispatch_file_method("close", &[file.clone()]).unwrap();
        assert_eq!(
            dispatch_file_method("is_closed", &[file.clone()]).unwrap(),
            Some(Value::Boolean(true))
        );
        assert!(dispatch_file_method("read_to_string", &[file]).is_err());

        dispatch_file_method("close", &[cloned]).unwrap();
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn file_handles_validate_sizes_and_open_options_types() {
        let path = temporary_path();
        let created = native_io_file_create(&[string(path.to_string_lossy().as_ref())]).unwrap();
        assert!(dispatch_file_method("read", &[created.clone(), Value::Integer(-1)]).is_err());
        assert!(dispatch_file_method("seek", &[created.clone(), Value::Integer(0), string("unknown")]).is_err());
        assert!(dispatch_open_options_method("read", &[native_io_open_options(&[]).unwrap(), Value::Integer(1)]).is_err());
        dispatch_file_method("close", &[created]).unwrap();
        fs::remove_file(path).unwrap();
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
    register_one(globals, "file_read_bytes", native_file_read_bytes);
    register_one(globals, "file_write", native_file_write);
    register_one(globals, "file_write_bytes", native_file_write_bytes);
    register_one(globals, "file_append", native_file_append);
    register_one(globals, "file_exists", native_file_exists);
    register_one(globals, "file_delete", native_file_delete);
    register_one(globals, "file_size", native_file_size);
    register_one(globals, "file_copy", native_file_copy);
    register_one(globals, "file_rename", native_file_rename);
    register_one(globals, "file_canonicalize", native_file_canonicalize);
    register_one(globals, "file_metadata", native_file_metadata);
    register_one(globals, "file_create_dir", native_file_create_dir);
    register_one(globals, "file_create_dir_all", native_file_create_dir_all);
    register_one(globals, "file_remove_dir", native_file_remove_dir);
    register_one(globals, "file_remove_dir_all", native_file_remove_dir_all);
    register_one(globals, "file_read_dir", native_file_read_dir);
    register_one(globals, "io_file_open", native_io_file_open);
    register_one(globals, "io_file_create", native_io_file_create);
    register_one(globals, "io_open_options", native_io_open_options);
}
