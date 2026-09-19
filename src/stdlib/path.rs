//! Manipulation de chemins de fichiers, native.
//!
//! Faisable en Kastel pur pour la manipulation textuelle simple, mais
//! gérer correctement les séparateurs `/` vs `\`, la normalisation et
//! les chemins absolus à la main serait fragile et non portable —
//! `std::path::Path` s'en charge déjà correctement.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value,
};

fn expect_string(value: &Value) -> Result<String, RuntimeError> {
    value.as_string_value().ok_or(RuntimeError::TypeError)
}

/// Joint tous les segments d'un tableau de chaînes en un seul chemin
/// (pas de varargs en Kastel — un tableau est le seul moyen propre
/// de passer un nombre variable de segments).
pub fn native_path_join(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let segments = match &args[0] {
        Value::Object(handle) => match &*handle.borrow() {
            crate::runtime::object::Object::Array(items) => items.clone(),
            crate::runtime::object::Object::Tuple(items) => items.clone(),
            _ => return Err(RuntimeError::TypeError),
        },
        _ => return Err(RuntimeError::TypeError),
    };

    let mut path = PathBuf::new();

    for segment in &segments {
        path.push(expect_string(segment)?);
    }

    Ok(Value::new_string(path.to_string_lossy().into_owned()))
}

pub fn native_path_exists(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;

    Ok(Value::Boolean(Path::new(&path).exists()))
}

pub fn native_path_is_dir(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;

    Ok(Value::Boolean(Path::new(&path).is_dir()))
}

pub fn native_path_is_file(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;

    Ok(Value::Boolean(Path::new(&path).is_file()))
}

pub fn native_path_absolute(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;

    let absolute = std::env::current_dir()
        .map_err(|error| RuntimeError::ModuleError(format!("path.absolute: {error}")))?
        .join(&path);

    Ok(Value::new_string(absolute.to_string_lossy().into_owned()))
}

pub fn native_path_basename(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;

    let name = Path::new(&path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();

    Ok(Value::new_string(name))
}

pub fn native_path_dirname(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;

    let parent = Path::new(&path)
        .parent()
        .map(|parent| parent.to_string_lossy().into_owned())
        .unwrap_or_default();

    Ok(Value::new_string(parent))
}

pub fn native_path_extension(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;

    let extension = Path::new(&path)
        .extension()
        .map(|extension| extension.to_string_lossy().into_owned())
        .unwrap_or_default();

    Ok(Value::new_string(extension))
}

pub fn native_path_stem(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let path = expect_string(&args[0])?;

    let stem = Path::new(&path)
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();

    Ok(Value::new_string(stem))
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
    register_one(globals, "path_join", native_path_join);
    register_one(globals, "path_exists", native_path_exists);
    register_one(globals, "path_is_dir", native_path_is_dir);
    register_one(globals, "path_is_file", native_path_is_file);
    register_one(globals, "path_absolute", native_path_absolute);
    register_one(globals, "path_basename", native_path_basename);
    register_one(globals, "path_dirname", native_path_dirname);
    register_one(globals, "path_extension", native_path_extension);
    register_one(globals, "path_stem", native_path_stem);
}

pub fn register_compiler(compiler: &mut Compiler) {
    define_one(compiler, "path_join");
    define_one(compiler, "path_exists");
    define_one(compiler, "path_is_dir");
    define_one(compiler, "path_is_file");
    define_one(compiler, "path_absolute");
    define_one(compiler, "path_basename");
    define_one(compiler, "path_dirname");
    define_one(compiler, "path_extension");
    define_one(compiler, "path_stem");
}
