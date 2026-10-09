//! Méthodes natives pour les lecteurs et écrivains tamponnés de `std.io`.

use std::collections::HashMap;
use std::io::{BufRead, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::{
    error::runtime_error::RuntimeError,
    runtime::{buffered_io as state, object::Object, value::Value},
};

fn expect_string(value: &Value) -> Result<String, RuntimeError> {
    value.as_string_value().ok_or(RuntimeError::TypeError)
}

fn capacity_from_value(value: &Value) -> Result<usize, RuntimeError> {
    let Value::Integer(value) = value else {
        return Err(RuntimeError::TypeError);
    };
    let capacity = usize::try_from(*value).map_err(|_| {
        RuntimeError::ModuleError("std.io buffer capacity must be a positive integer".to_string())
    })?;
    if capacity == 0 || capacity > state::MAX_BUFFER_CAPACITY {
        return Err(RuntimeError::ModuleError(format!(
            "std.io buffer capacity must be between 1 and {} bytes",
            state::MAX_BUFFER_CAPACITY
        )));
    }
    Ok(capacity)
}

fn io_error(kind: &str, path: &Path, error: std::io::Error) -> RuntimeError {
    RuntimeError::ModuleError(format!("std.io.{kind}: {}: {error}", path.display()))
}

fn closed_error(kind: &str, path: &Path) -> RuntimeError {
    RuntimeError::ModuleError(format!(
        "std.io.{kind}: {}: handle is closed",
        path.display()
    ))
}

fn method_arity(args: &[Value], expected: usize) -> Result<(), RuntimeError> {
    let found = args.len().saturating_sub(1);
    if args.is_empty() || found != expected {
        return Err(RuntimeError::WrongArgumentCount { expected, found });
    }
    Ok(())
}

fn reader_handle(
    value: &Value,
) -> Result<std::rc::Rc<std::cell::RefCell<state::BufReaderState>>, RuntimeError> {
    let Value::Object(handle) = value else {
        return Err(RuntimeError::TypeError);
    };
    match &*handle.borrow() {
        Object::BufReader(reader) => Ok(reader.clone()),
        _ => Err(RuntimeError::TypeError),
    }
}

fn writer_handle(
    value: &Value,
) -> Result<std::rc::Rc<std::cell::RefCell<state::BufWriterState>>, RuntimeError> {
    let Value::Object(handle) = value else {
        return Err(RuntimeError::TypeError);
    };
    match &*handle.borrow() {
        Object::BufWriter(writer) => Ok(writer.clone()),
        _ => Err(RuntimeError::TypeError),
    }
}

fn bytes_value(bytes: Vec<u8>) -> Value {
    Value::new_array(
        bytes
            .into_iter()
            .map(|byte| Value::Integer(i64::from(byte)))
            .collect(),
    )
}

fn bytes_from_value(value: &Value) -> Result<Vec<u8>, RuntimeError> {
    if let Some(text) = value.as_string_value() {
        return Ok(text.into_bytes());
    }
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

/// Lit une ligne au plus `MAX_READ_SIZE` octets. Une ligne trop longue est
/// consommée jusqu'au prochain '\n' puis signalée comme erreur pour que le
/// prochain appel reparte sur une frontière de ligne cohérente.
fn read_bounded_line(
    reader: &mut std::io::BufReader<std::fs::File>,
    path: &Path,
) -> Result<Option<String>, RuntimeError> {
    let mut bytes = Vec::new();
    let mut saw_any = false;
    let mut too_long = false;

    loop {
        let (consume, newline, eof) = {
            let buffer = reader
                .fill_buf()
                .map_err(|error| io_error("BufReader.read_line", path, error))?;
            if buffer.is_empty() {
                (0, false, true)
            } else {
                let newline_position = buffer.iter().position(|byte| *byte == b'\n');
                let to_consume = newline_position.map_or(buffer.len(), |index| index + 1);
                saw_any = true;
                if !too_long {
                    let remaining = state::MAX_READ_SIZE.saturating_sub(bytes.len());
                    let to_copy = to_consume.min(remaining);
                    bytes.try_reserve(to_copy).map_err(|error| {
                        RuntimeError::ModuleError(format!(
                            "std.io.BufReader.read_line: cannot reserve line buffer: {error}"
                        ))
                    })?;
                    bytes.extend_from_slice(&buffer[..to_copy]);
                    if to_copy < to_consume {
                        too_long = true;
                    }
                }
                (to_consume, newline_position.is_some(), false)
            }
        };

        if eof {
            break;
        }
        reader.consume(consume);
        if newline {
            break;
        }
    }

    if too_long {
        return Err(RuntimeError::ModuleError(format!(
            "std.io.BufReader.read_line: line exceeds {} bytes",
            state::MAX_READ_SIZE
        )));
    }
    if !saw_any {
        return Ok(None);
    }
    String::from_utf8(bytes).map(Some).map_err(|error| {
        RuntimeError::ModuleError(format!(
            "std.io.BufReader.read_line: line is not valid UTF-8: {error}"
        ))
    })
}

pub fn native_io_buf_reader(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }
    let path = expect_string(&args[0])?;
    let reader = state::BufReaderState::open(Path::new(&path), state::DEFAULT_BUFFER_CAPACITY)
        .map_err(|error| io_error("buf_reader", Path::new(&path), error))?;
    Ok(Value::new_buf_reader(reader))
}

pub fn native_io_buf_reader_with_capacity(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }
    let path = expect_string(&args[0])?;
    let capacity = capacity_from_value(&args[1])?;
    let reader = state::BufReaderState::open(Path::new(&path), capacity)
        .map_err(|error| io_error("buf_reader_with_capacity", Path::new(&path), error))?;
    Ok(Value::new_buf_reader(reader))
}

/// Lit jusqu'à la fin du flux, avec un plafond explicite par appel.
/// Pour les fichiers ordinaires, vérifie la taille restante avant de consommer
/// des octets ; la boucle applique aussi la limite si le fichier grossit ensuite.
fn read_bounded_to_end(
    reader: &mut std::io::BufReader<std::fs::File>,
    path: &Path,
) -> Result<Vec<u8>, RuntimeError> {
    let position = reader
        .stream_position()
        .map_err(|error| io_error("BufReader.read_to_end.position", path, error))?;
    let file_length = reader
        .get_ref()
        .metadata()
        .map_err(|error| io_error("BufReader.read_to_end.metadata", path, error))?
        .len();
    let remaining = file_length.saturating_sub(position);
    if remaining > state::MAX_READ_SIZE as u64 {
        return Err(RuntimeError::ModuleError(format!(
            "std.io.BufReader.read_to_end: remaining data exceeds {} bytes; use read(size) in chunks",
            state::MAX_READ_SIZE
        )));
    }

    let mut output = Vec::new();
    let mut chunk = [0_u8; 8192];

    loop {
        if output.len() == state::MAX_READ_SIZE {
            let buffered = reader
                .fill_buf()
                .map_err(|error| io_error("BufReader.read_to_end", path, error))?;
            if buffered.is_empty() {
                return Ok(output);
            }
            return Err(RuntimeError::ModuleError(format!(
                "std.io.BufReader.read_to_end: remaining data exceeds {} bytes; use read(size) in chunks",
                state::MAX_READ_SIZE
            )));
        }

        let amount = chunk.len().min(state::MAX_READ_SIZE - output.len());
        output.try_reserve(amount).map_err(|error| {
            RuntimeError::ModuleError(format!(
                "std.io.BufReader.read_to_end: allocation failed: {error}"
            ))
        })?;
        let count = reader
            .read(&mut chunk[..amount])
            .map_err(|error| io_error("BufReader.read_to_end", path, error))?;
        if count == 0 {
            return Ok(output);
        }
        output.extend_from_slice(&chunk[..count]);
    }
}

pub fn native_io_buf_writer_append(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }
    let path = expect_string(&args[0])?;
    let writer = state::BufWriterState::append(Path::new(&path), state::DEFAULT_BUFFER_CAPACITY)
        .map_err(|error| io_error("buf_writer_append", Path::new(&path), error))?;
    Ok(Value::new_buf_writer(writer))
}

pub fn native_io_buf_writer_append_with_capacity(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }
    let path = expect_string(&args[0])?;
    let capacity = capacity_from_value(&args[1])?;
    let writer = state::BufWriterState::append(Path::new(&path), capacity)
        .map_err(|error| io_error("buf_writer_append_with_capacity", Path::new(&path), error))?;
    Ok(Value::new_buf_writer(writer))
}

pub fn native_io_buf_writer(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }
    let path = expect_string(&args[0])?;
    let writer = state::BufWriterState::create(Path::new(&path), state::DEFAULT_BUFFER_CAPACITY)
        .map_err(|error| io_error("buf_writer", Path::new(&path), error))?;
    Ok(Value::new_buf_writer(writer))
}

pub fn native_io_buf_writer_with_capacity(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }
    let path = expect_string(&args[0])?;
    let capacity = capacity_from_value(&args[1])?;
    let writer = state::BufWriterState::create(Path::new(&path), capacity)
        .map_err(|error| io_error("buf_writer_with_capacity", Path::new(&path), error))?;
    Ok(Value::new_buf_writer(writer))
}

pub fn dispatch_buf_reader_method(
    method: &str,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    const METHODS: &[&str] = &[
        "read",
        "read_line",
        "read_to_end",
        "read_to_string",
        "buffer",
        "seek",
        "stream_position",
        "close",
        "is_closed",
        "path",
        "buffer_capacity",
    ];
    if !METHODS.contains(&method) {
        return Ok(None);
    }
    if args.is_empty() {
        return Err(RuntimeError::TypeError);
    }
    let handle = reader_handle(&args[0])?;
    if method == "is_closed" {
        method_arity(args, 0)?;
        return Ok(Some(Value::Boolean(handle.borrow().is_closed())));
    }
    if method == "path" {
        method_arity(args, 0)?;
        return Ok(Some(Value::new_string(
            handle.borrow().path.to_string_lossy().into_owned(),
        )));
    }
    if method == "buffer_capacity" {
        method_arity(args, 0)?;
        let capacity = handle.borrow().capacity();
        return Ok(Some(Value::Integer(i64::try_from(capacity).map_err(
            |_| {
                RuntimeError::ModuleError(
                    "buffer capacity exceeds Kastel integer range".to_string(),
                )
            },
        )?)));
    }
    if method == "close" {
        method_arity(args, 0)?;
        handle.borrow_mut().close();
        return Ok(Some(Value::None));
    }

    let path = handle.borrow().path.clone();
    if method == "read" {
        method_arity(args, 1)?;
        let Value::Integer(size) = &args[1] else {
            return Err(RuntimeError::TypeError);
        };
        let size = usize::try_from(*size).map_err(|_| {
            RuntimeError::ModuleError(
                "std.io.BufReader.read: size must be non-negative".to_string(),
            )
        })?;
        if size > state::MAX_READ_SIZE {
            return Err(RuntimeError::ModuleError(format!(
                "std.io.BufReader.read: size exceeds {} bytes",
                state::MAX_READ_SIZE
            )));
        }
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(size).map_err(|error| {
            RuntimeError::ModuleError(format!("std.io.BufReader.read: allocation failed: {error}"))
        })?;
        bytes.resize(size, 0);
        let mut state_ref = handle.borrow_mut();
        let reader = state_ref
            .reader
            .as_mut()
            .ok_or_else(|| closed_error("BufReader.read", &path))?;
        let count = reader
            .read(&mut bytes)
            .map_err(|error| io_error("BufReader.read", &path, error))?;
        bytes.truncate(count);
        return Ok(Some(bytes_value(bytes)));
    }

    if method == "read_line" {
        method_arity(args, 0)?;
        let mut state_ref = handle.borrow_mut();
        let reader = state_ref
            .reader
            .as_mut()
            .ok_or_else(|| closed_error("BufReader.read_line", &path))?;
        let line = read_bounded_line(reader, &path)?;
        return Ok(Some(Value::new_option(line.map(Value::new_string))));
    }

    if method == "read_to_end" || method == "read_to_string" {
        method_arity(args, 0)?;
        let mut state_ref = handle.borrow_mut();
        let reader = state_ref
            .reader
            .as_mut()
            .ok_or_else(|| closed_error(&format!("BufReader.{method}"), &path))?;
        let bytes = read_bounded_to_end(reader, &path)?;
        if method == "read_to_end" {
            return Ok(Some(bytes_value(bytes)));
        }
        let text = String::from_utf8(bytes).map_err(|error| {
            RuntimeError::ModuleError(format!(
                "std.io.BufReader.read_to_string: remaining data is not valid UTF-8: {error}"
            ))
        })?;
        return Ok(Some(Value::new_string(text)));
    }

    if method == "buffer" {
        method_arity(args, 0)?;
        let mut state_ref = handle.borrow_mut();
        let reader = state_ref
            .reader
            .as_mut()
            .ok_or_else(|| closed_error("BufReader.buffer", &path))?;
        let buffered = reader
            .fill_buf()
            .map_err(|error| io_error("BufReader.buffer", &path, error))?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(buffered.len()).map_err(|error| {
            RuntimeError::ModuleError(format!(
                "std.io.BufReader.buffer: allocation failed: {error}"
            ))
        })?;
        bytes.extend_from_slice(buffered);
        return Ok(Some(bytes_value(bytes)));
    }

    if method == "seek" {
        method_arity(args, 2)?;
        let Value::Integer(offset) = &args[1] else {
            return Err(RuntimeError::TypeError);
        };
        let offset = *offset;
        let origin = match &args[2] {
            Value::Object(handle) => {
                let object = handle.borrow();
                match &*object {
                    Object::EnumVariant {
                        enum_name,
                        variant_name,
                        ..
                    } if enum_name == "SeekFrom" => variant_name.clone(),
                    Object::String(value) => value.clone(),
                    _ => return Err(RuntimeError::TypeError),
                }
            }
            _ => return Err(RuntimeError::TypeError),
        };
        let seek_from = match origin.as_str() {
            "Start" | "start" => SeekFrom::Start(u64::try_from(offset).map_err(|_| {
                RuntimeError::ModuleError(
                    "std.io.BufReader.seek: Start offset must be non-negative".to_string(),
                )
            })?),
            "Current" | "current" => SeekFrom::Current(offset),
            "End" | "end" => SeekFrom::End(offset),
            _ => return Err(RuntimeError::TypeError),
        };
        let mut state_ref = handle.borrow_mut();
        let reader = state_ref
            .reader
            .as_mut()
            .ok_or_else(|| closed_error("BufReader.seek", &path))?;
        let position = reader
            .seek(seek_from)
            .map_err(|error| io_error("BufReader.seek", &path, error))?;
        return Ok(Some(Value::Integer(i64::try_from(position).map_err(
            |_| {
                RuntimeError::ModuleError(
                    "std.io.BufReader.seek: position exceeds Kastel integer range".to_string(),
                )
            },
        )?)));
    }

    if method == "stream_position" {
        method_arity(args, 0)?;
        let mut state_ref = handle.borrow_mut();
        let reader = state_ref
            .reader
            .as_mut()
            .ok_or_else(|| closed_error("BufReader.stream_position", &path))?;
        let position = reader
            .stream_position()
            .map_err(|error| io_error("BufReader.stream_position", &path, error))?;
        return Ok(Some(Value::Integer(i64::try_from(position).map_err(
            |_| {
                RuntimeError::ModuleError(
                    "std.io.BufReader.stream_position: position exceeds Kastel integer range"
                        .to_string(),
                )
            },
        )?)));
    }

    Ok(None)
}

pub fn dispatch_buf_writer_method(
    method: &str,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    const METHODS: &[&str] = &[
        "write",
        "write_all",
        "flush",
        "sync_all",
        "close",
        "is_closed",
        "path",
        "buffer_capacity",
    ];
    if !METHODS.contains(&method) {
        return Ok(None);
    }
    if args.is_empty() {
        return Err(RuntimeError::TypeError);
    }
    let handle = writer_handle(&args[0])?;
    if method == "is_closed" {
        method_arity(args, 0)?;
        return Ok(Some(Value::Boolean(handle.borrow().is_closed())));
    }
    if method == "path" {
        method_arity(args, 0)?;
        return Ok(Some(Value::new_string(
            handle.borrow().path.to_string_lossy().into_owned(),
        )));
    }
    if method == "buffer_capacity" {
        method_arity(args, 0)?;
        let capacity = handle.borrow().capacity();
        return Ok(Some(Value::Integer(i64::try_from(capacity).map_err(
            |_| {
                RuntimeError::ModuleError(
                    "buffer capacity exceeds Kastel integer range".to_string(),
                )
            },
        )?)));
    }
    let path = handle.borrow().path.clone();
    if method == "close" {
        method_arity(args, 0)?;
        handle
            .borrow_mut()
            .close()
            .map_err(|error| io_error("BufWriter.close", &path, error))?;
        return Ok(Some(Value::None));
    }
    if method == "flush" {
        method_arity(args, 0)?;
        let mut state_ref = handle.borrow_mut();
        let writer = state_ref
            .writer
            .as_mut()
            .ok_or_else(|| closed_error("BufWriter.flush", &path))?;
        writer
            .flush()
            .map_err(|error| io_error("BufWriter.flush", &path, error))?;
        return Ok(Some(Value::None));
    }
    if method == "sync_all" {
        method_arity(args, 0)?;
        let mut state_ref = handle.borrow_mut();
        let writer = state_ref
            .writer
            .as_mut()
            .ok_or_else(|| closed_error("BufWriter.sync_all", &path))?;
        writer
            .flush()
            .map_err(|error| io_error("BufWriter.sync_all.flush", &path, error))?;
        writer
            .get_ref()
            .sync_all()
            .map_err(|error| io_error("BufWriter.sync_all", &path, error))?;
        return Ok(Some(Value::None));
    }
    if method == "write" || method == "write_all" {
        method_arity(args, 1)?;
        let bytes = bytes_from_value(&args[1])?;
        let mut state_ref = handle.borrow_mut();
        let writer = state_ref
            .writer
            .as_mut()
            .ok_or_else(|| closed_error("BufWriter.write", &path))?;
        if method == "write_all" {
            writer
                .write_all(&bytes)
                .map_err(|error| io_error("BufWriter.write_all", &path, error))?;
            return Ok(Some(Value::None));
        }
        let count = writer
            .write(&bytes)
            .map_err(|error| io_error("BufWriter.write", &path, error))?;
        return Ok(Some(Value::Integer(i64::try_from(count).map_err(
            |_| {
                RuntimeError::ModuleError(
                    "std.io.BufWriter.write: byte count exceeds Kastel integer range".to_string(),
                )
            },
        )?)));
    }
    Ok(None)
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "io_buf_reader".to_string(),
        Value::NativeFunction(native_io_buf_reader),
    );
    globals.insert(
        "io_buf_reader_with_capacity".to_string(),
        Value::NativeFunction(native_io_buf_reader_with_capacity),
    );
    globals.insert(
        "io_buf_writer".to_string(),
        Value::NativeFunction(native_io_buf_writer),
    );
    globals.insert(
        "io_buf_writer_with_capacity".to_string(),
        Value::NativeFunction(native_io_buf_writer_with_capacity),
    );
    globals.insert(
        "io_buf_writer_append".to_string(),
        Value::NativeFunction(native_io_buf_writer_append),
    );
    globals.insert(
        "io_buf_writer_append_with_capacity".to_string(),
        Value::NativeFunction(native_io_buf_writer_append_with_capacity),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(label: &str) -> String {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir()
            .join(format!(
                "kastel-io-{label}-{}-{stamp}.tmp",
                std::process::id()
            ))
            .to_string_lossy()
            .into_owned()
    }

    fn string(value: &str) -> Value {
        Value::new_string(value.to_string())
    }

    fn integer_list(value: &Value) -> Vec<i64> {
        let Value::Object(handle) = value else {
            panic!("expected list")
        };
        let object = handle.borrow();
        let Object::Array(values) = &*object else {
            panic!("expected array object")
        };
        values
            .iter()
            .map(|value| match value {
                Value::Integer(n) => *n,
                _ => panic!("expected integer"),
            })
            .collect()
    }

    #[test]
    fn buffered_reader_reads_bytes_lines_seek_and_eof_option() {
        let path = temp_path("reader");
        std::fs::write(&path, b"first\nsecond").unwrap();
        let reader = native_io_buf_reader(&[string(&path)]).unwrap();
        assert_eq!(
            dispatch_buf_reader_method("buffer_capacity", &[reader.clone()])
                .unwrap()
                .unwrap(),
            Value::Integer(state::DEFAULT_BUFFER_CAPACITY as i64)
        );
        let buffered = dispatch_buf_reader_method("buffer", &[reader.clone()])
            .unwrap()
            .unwrap();
        assert_eq!(
            integer_list(&buffered),
            b"first\nsecond"
                .iter()
                .map(|byte| i64::from(*byte))
                .collect::<Vec<_>>()
        );
        let line1 = dispatch_buf_reader_method("read_line", &[reader.clone()])
            .unwrap()
            .unwrap();
        let line2 = dispatch_buf_reader_method("read_line", &[reader.clone()])
            .unwrap()
            .unwrap();
        assert_eq!(line1.to_string(), "Some(first\n)");
        assert_eq!(line2.to_string(), "Some(second)");
        let eof = dispatch_buf_reader_method("read_line", &[reader.clone()])
            .unwrap()
            .unwrap();
        assert_eq!(eof.to_string(), "None");
        dispatch_buf_reader_method(
            "seek",
            &[
                reader.clone(),
                Value::Integer(0),
                Value::new_enum_variant(
                    "SeekFrom".to_string(),
                    "Start".to_string(),
                    std::collections::HashMap::new(),
                ),
            ],
        )
        .unwrap();
        let bytes = dispatch_buf_reader_method("read", &[reader.clone(), Value::Integer(5)])
            .unwrap()
            .unwrap();
        assert_eq!(integer_list(&bytes), vec![102, 105, 114, 115, 116]);
        dispatch_buf_reader_method("close", &[reader.clone()]).unwrap();
        assert!(dispatch_buf_reader_method("read", &[reader, Value::Integer(1)]).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn buffered_writer_flushes_and_close_is_idempotent() {
        let path = temp_path("writer");
        let writer = native_io_buf_writer(&[string(&path)]).unwrap();
        dispatch_buf_writer_method("write", &[writer.clone(), string("hello")]).unwrap();
        dispatch_buf_writer_method("flush", &[writer.clone()]).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello");
        dispatch_buf_writer_method(
            "write_all",
            &[
                writer.clone(),
                Value::new_array(vec![Value::Integer(32), Value::Integer(33)]),
            ],
        )
        .unwrap();
        dispatch_buf_writer_method("close", &[writer.clone()]).unwrap();
        dispatch_buf_writer_method("close", &[writer.clone()]).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"hello !");
        assert!(dispatch_buf_writer_method("flush", &[writer]).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn buffered_reader_reads_to_end_and_to_string() {
        let path = temp_path("reader-all");
        std::fs::write(&path, "bonjour 🌍".as_bytes()).unwrap();

        let reader = native_io_buf_reader(&[string(&path)]).unwrap();
        let text = dispatch_buf_reader_method("read_to_string", &[reader.clone()])
            .unwrap()
            .unwrap();
        assert_eq!(text.as_string_value().as_deref(), Some("bonjour 🌍"));
        dispatch_buf_reader_method("close", &[reader]).unwrap();

        let reader = native_io_buf_reader(&[string(&path)]).unwrap();
        let bytes = dispatch_buf_reader_method("read_to_end", &[reader.clone()])
            .unwrap()
            .unwrap();
        assert_eq!(
            integer_list(&bytes),
            "bonjour 🌍"
                .as_bytes()
                .iter()
                .map(|byte| i64::from(*byte))
                .collect::<Vec<_>>()
        );
        dispatch_buf_reader_method("close", &[reader]).unwrap();
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn buffered_writer_append_keeps_existing_file_contents() {
        let path = temp_path("append");
        std::fs::write(&path, b"before").unwrap();
        let writer =
            native_io_buf_writer_append_with_capacity(&[string(&path), Value::Integer(32)])
                .unwrap();
        dispatch_buf_writer_method("write_all", &[writer.clone(), string("+after")]).unwrap();
        dispatch_buf_writer_method("close", &[writer]).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"before+after");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn constructors_reject_invalid_capacities_and_wrong_shapes() {
        let path = temp_path("capacity");
        assert!(native_io_buf_reader_with_capacity(&[string(&path), Value::Integer(0)]).is_err());
        assert!(native_io_buf_writer_with_capacity(&[string(&path), Value::Integer(-1)]).is_err());
        assert!(native_io_buf_reader(&[string(&path), Value::Integer(1)]).is_err());
        assert!(native_io_buf_writer(&[string(&path), Value::Integer(1)]).is_err());
        assert!(
            native_io_buf_writer_with_capacity(&[string(&path), Value::Boolean(true)]).is_err()
        );
    }

    #[test]
    fn buffered_reader_rejects_invalid_utf8_for_read_to_string() {
        let path = temp_path("invalid-utf8");
        std::fs::write(&path, [0x66, 0x80]).unwrap();
        let reader = native_io_buf_reader(&[string(&path)]).unwrap();
        assert!(dispatch_buf_reader_method("read_to_string", &[reader.clone()]).is_err());
        dispatch_buf_reader_method("close", &[reader]).unwrap();
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn buffered_reader_enforces_read_to_end_limit() {
        let path = temp_path("reader-limit");
        let contents = vec![b'x'; state::MAX_READ_SIZE + 1];
        std::fs::write(&path, contents).unwrap();
        let reader =
            native_io_buf_reader_with_capacity(&[string(&path), Value::Integer(8192)]).unwrap();
        assert!(dispatch_buf_reader_method("read_to_end", &[reader.clone()]).is_err());
        // A regular-file size precheck rejects the read without consuming bytes.
        let first = dispatch_buf_reader_method("read", &[reader.clone(), Value::Integer(1)])
            .unwrap()
            .unwrap();
        assert_eq!(integer_list(&first), vec![i64::from(b'x')]);
        dispatch_buf_reader_method("close", &[reader]).unwrap();
        std::fs::remove_file(path).unwrap();
    }
}
