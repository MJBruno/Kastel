//! Exécution de processus externes, native.
//!
//! `os.rs` documentait volontairement l'absence de `system(...)` (surface
//! d'injection de commande shell). Ce module répond au besoin réel
//! (lancer un programme, récupérer stdout/stderr/code) SANS shell :
//! `std::process::Command::new(program).args(arguments)` exécute le
//! programme directement, sans passer par `/bin/sh -c "..."`, donc sans
//! interpolation shell ni risque d'injection — chaque argument reste une
//! chaîne opaque passée telle quelle au programme.
//!
//! Reste hors de portée volontairement (à ajouter séparément si un
//! besoin réel se présente) : pipes vers un processus déjà lancé,
//! entrée standard, exécution asynchrone/en arrière-plan, timeout.

use std::collections::HashMap;
use std::process::Command;

use crate::{
    error::runtime_error::RuntimeError, runtime::value::Value,
};

fn expect_string(value: &Value) -> Result<String, RuntimeError> {
    value.as_string_value().ok_or(RuntimeError::TypeError)
}

fn expect_string_array(value: &Value) -> Result<Vec<String>, RuntimeError> {
    match value {
        Value::Object(handle) => match &*handle.borrow() {
            crate::runtime::object::Object::Array(items) => {
                items.iter().map(expect_string).collect()
            }
            crate::runtime::object::Object::Tuple(items) => {
                items.iter().map(expect_string).collect()
            }
            _ => Err(RuntimeError::TypeError),
        },
        _ => Err(RuntimeError::TypeError),
    }
}

fn process_error(program: &str, error: std::io::Error) -> RuntimeError {
    RuntimeError::ModuleError(format!("process.run: {program}: {error}"))
}

/// `process_run(program, arguments)` : lance `program` avec la liste
/// d'arguments donnée (pas de shell), attend qu'il se termine, et
/// renvoie `{ stdout: str, stderr: str, code: int, success: bool }`.
///
/// Un `code` de -1 signale un processus terminé par un signal (pas de
/// code de sortie côté Unix) plutôt qu'un vrai code de sortie 255 —
/// distinct et sans ambiguïté avec un `exit(255)` explicite du programme.
///
/// Lève une exception uniquement si le programme n'a même pas pu être
/// lancé (introuvable, permissions, ...) : un exit code non nul reste un
/// résultat normal, à charge de l'appelant Kastel de vérifier `success`.
pub fn native_process_run(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let program = expect_string(&args[0])?;
    let arguments = expect_string_array(&args[1])?;

    let output = Command::new(&program)
        .args(&arguments)
        .output()
        .map_err(|error| process_error(&program, error))?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code().unwrap_or(-1) as i64;

    Ok(Value::new_record(vec![
        ("stdout".to_string(), Value::new_string(stdout)),
        ("stderr".to_string(), Value::new_string(stderr)),
        ("code".to_string(), Value::Integer(code)),
        (
            "success".to_string(),
            Value::Boolean(output.status.success()),
        ),
    ]))
}

fn register_one(globals: &mut HashMap<String, Value>, name: &str, function: super::NativeFn) {
    globals.insert(name.to_string(), Value::NativeFunction(function));
}

pub fn register(globals: &mut HashMap<String, Value>) {
    register_one(globals, "process_run", native_process_run);
}

