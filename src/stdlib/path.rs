//! Manipulation de chemins de fichiers, native.
//!
//! Faisable en Kastel pur pour la manipulation textuelle simple, mais
//! gérer correctement les séparateurs `/` vs `\`, la normalisation et
//! les chemins absolus à la main serait fragile et non portable —
//! `std::path::Path` s'en charge déjà correctement.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{
    error::runtime_error::RuntimeError, runtime::value::Value,
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


pub fn native_path_is_absolute(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount { expected: 1, found: args.len() });
    }
    let path = expect_string(&args[0])?;
    Ok(Value::Boolean(Path::new(&path).is_absolute()))
}

pub fn native_path_is_relative(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount { expected: 1, found: args.len() });
    }
    let path = expect_string(&args[0])?;
    Ok(Value::Boolean(Path::new(&path).is_relative()))
}

pub fn native_path_has_root(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount { expected: 1, found: args.len() });
    }
    let path = expect_string(&args[0])?;
    Ok(Value::Boolean(Path::new(&path).has_root()))
}

pub fn native_path_starts_with(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount { expected: 2, found: args.len() });
    }
    let path = expect_string(&args[0])?;
    let base = expect_string(&args[1])?;
    Ok(Value::Boolean(Path::new(&path).starts_with(Path::new(&base))))
}

pub fn native_path_ends_with(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount { expected: 2, found: args.len() });
    }
    let path = expect_string(&args[0])?;
    let suffix = expect_string(&args[1])?;
    Ok(Value::Boolean(Path::new(&path).ends_with(Path::new(&suffix))))
}

pub fn native_path_strip_prefix(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount { expected: 2, found: args.len() });
    }
    let path = expect_string(&args[0])?;
    let prefix = expect_string(&args[1])?;
    let stripped = Path::new(&path)
        .strip_prefix(Path::new(&prefix))
        .map_err(|error| RuntimeError::ModuleError(format!("path.strip_prefix: {error}")))?;
    Ok(Value::new_string(stripped.to_string_lossy().into_owned()))
}

pub fn native_path_with_extension(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount { expected: 2, found: args.len() });
    }
    let path = expect_string(&args[0])?;
    let extension = expect_string(&args[1])?;
    Ok(Value::new_string(
        Path::new(&path).with_extension(extension).to_string_lossy().into_owned(),
    ))
}

pub fn native_path_with_file_name(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount { expected: 2, found: args.len() });
    }
    let path = expect_string(&args[0])?;
    let file_name = expect_string(&args[1])?;
    Ok(Value::new_string(
        Path::new(&path).with_file_name(file_name).to_string_lossy().into_owned(),
    ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::object::Object;

    fn string(value: &str) -> Value {
        Value::new_string(value.to_string())
    }

    fn strings(values: &[&str]) -> Vec<Value> {
        values.iter().map(|value| string(value)).collect()
    }

    #[test]
    fn path_join_accepts_list_and_tuple_segments() {
        let list = Value::new_array(strings(&["tmp", "kastel", "demo.ks"]));
        let tuple = Value::new_tuple(strings(&["tmp", "kastel", "demo.ks"]));

        let expected = PathBuf::from("tmp").join("kastel").join("demo.ks");
        let expected = expected.to_string_lossy().into_owned();

        assert_eq!(
            native_path_join(&[list]).unwrap(),
            string(&expected)
        );
        assert_eq!(
            native_path_join(&[tuple]).unwrap(),
            string(&expected)
        );
    }

    #[test]
    fn path_metadata_helpers_match_rust_path_semantics() {
        let path = if cfg!(windows) {
            r"tmp\kastel\demo.test.ks"
        } else {
            "tmp/kastel/demo.test.ks"
        };

        assert_eq!(native_path_basename(&[string(path)]).unwrap(), string("demo.test.ks"));
        assert_eq!(native_path_extension(&[string(path)]).unwrap(), string("ks"));
        assert_eq!(native_path_stem(&[string(path)]).unwrap(), string("demo.test"));

        let dirname = native_path_dirname(&[string(path)]).unwrap();
        let expected_parent = Path::new(path)
            .parent()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default();
        assert_eq!(dirname, string(&expected_parent));
    }

    #[test]
    fn path_predicates_and_absolute_are_consistent() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(format!(
            "kastel_stdlib_path_test_{}",
            std::process::id()
        ));

        std::fs::create_dir_all(&path).unwrap();
        let file = path.join("item.ks");
        std::fs::write(&file, "ok").unwrap();

        let dir_value = string(path.to_string_lossy().as_ref());
        let file_value = string(file.to_string_lossy().as_ref());

        assert_eq!(native_path_exists(&[dir_value.clone()]).unwrap(), Value::Boolean(true));
        assert_eq!(native_path_is_dir(&[dir_value.clone()]).unwrap(), Value::Boolean(true));
        assert_eq!(native_path_is_file(&[dir_value]).unwrap(), Value::Boolean(false));
        assert_eq!(native_path_is_file(&[file_value.clone()]).unwrap(), Value::Boolean(true));
        assert_eq!(native_path_is_dir(&[file_value.clone()]).unwrap(), Value::Boolean(false));

        let absolute = native_path_absolute(&[file_value]).unwrap();
        let Value::Object(handle) = absolute else {
            panic!("path_absolute() must return a string")
        };
        let Object::String(value) = &*handle.borrow() else {
            panic!("path_absolute() must return a string")
        };
        assert!(Path::new(value).is_absolute());

        std::fs::remove_file(&file).unwrap();
        std::fs::remove_dir_all(&path).unwrap();
    }

    #[test]
    fn path_platform_predicates_and_transformations_match_rust() {
        let path = if cfg!(windows) { r"tmp\kastel\demo.test.ks" } else { "tmp/kastel/demo.test.ks" };
        let absolute = if cfg!(windows) { r"C:\tmp\kastel\demo.test.ks" } else { "/tmp/kastel/demo.test.ks" };

        assert_eq!(native_path_is_relative(&[string(path)]).unwrap(), Value::Boolean(true));
        assert_eq!(native_path_is_absolute(&[string(path)]).unwrap(), Value::Boolean(false));
        assert_eq!(native_path_has_root(&[string(path)]).unwrap(), Value::Boolean(false));
        assert_eq!(native_path_is_absolute(&[string(absolute)]).unwrap(), Value::Boolean(true));
        assert_eq!(native_path_has_root(&[string(absolute)]).unwrap(), Value::Boolean(true));

        assert_eq!(
            native_path_starts_with(&[string(path), string(if cfg!(windows) { r"tmp\kastel" } else { "tmp/kastel" })]).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            native_path_ends_with(&[string(path), string("demo.test.ks")]).unwrap(),
            Value::Boolean(true)
        );

        let stripped = native_path_strip_prefix(&[
            string(path),
            string(if cfg!(windows) { r"tmp\kastel" } else { "tmp/kastel" }),
        ]).unwrap();
        assert_eq!(stripped, string("demo.test.ks"));

        let changed_extension = native_path_with_extension(&[string(path), string("txt")]).unwrap();
        assert_eq!(native_path_extension(&[changed_extension]).unwrap(), string("txt"));

        let changed_name = native_path_with_file_name(&[string(path), string("other.bin")]).unwrap();
        assert_eq!(native_path_basename(&[changed_name]).unwrap(), string("other.bin"));
    }

    #[test]
    fn path_operations_reject_wrong_shapes() {
        assert!(matches!(
            native_path_join(&[Value::Integer(1)]),
            Err(RuntimeError::TypeError)
        ));
        assert!(matches!(
            native_path_exists(&[]),
            Err(RuntimeError::WrongArgumentCount { expected: 1, found: 0 })
        ));
        assert!(matches!(
            native_path_basename(&[Value::Integer(1)]),
            Err(RuntimeError::TypeError)
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
    register_one(globals, "path_join", native_path_join);
    register_one(globals, "path_exists", native_path_exists);
    register_one(globals, "path_is_dir", native_path_is_dir);
    register_one(globals, "path_is_file", native_path_is_file);
    register_one(globals, "path_is_absolute", native_path_is_absolute);
    register_one(globals, "path_is_relative", native_path_is_relative);
    register_one(globals, "path_has_root", native_path_has_root);
    register_one(globals, "path_starts_with", native_path_starts_with);
    register_one(globals, "path_ends_with", native_path_ends_with);
    register_one(globals, "path_absolute", native_path_absolute);
    register_one(globals, "path_basename", native_path_basename);
    register_one(globals, "path_dirname", native_path_dirname);
    register_one(globals, "path_extension", native_path_extension);
    register_one(globals, "path_stem", native_path_stem);
    register_one(globals, "path_strip_prefix", native_path_strip_prefix);
    register_one(globals, "path_with_extension", native_path_with_extension);
    register_one(globals, "path_with_file_name", native_path_with_file_name);
}

