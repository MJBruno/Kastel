//! Record : `{ name: "Bruno", age: 25 }`.
//!
//! Champs nommés (identifiants), forme FIXE. Les champs se lisent et
//! s'écrivent par `p.name` ; ce module ne fournit que quelques méthodes
//! d'introspection. Si un champ porte le même nom qu'une de ces méthodes,
//! c'est le CHAMP qui gagne (voir `VirtualMachine::op_invoke_method`) : un
//! record peut donc contenir des fonctions (`{ greet: func() { ... } }`).

use crate::error::runtime_error::RuntimeError;
use crate::runtime::value::Value;

fn expect_receiver(args: &[Value], expected: usize) -> Result<(), RuntimeError> {
    if args.len() != expected {
        return Err(RuntimeError::WrongArgumentCount {
            expected,
            found: args.len(),
        });
    }

    Ok(())
}

/// `keys()` -> liste des noms de champs, dans l'ordre de déclaration.
pub fn native_keys(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_receiver(args, 1)?;

    let names = args[0]
        .record_fields()?
        .into_iter()
        .map(|(name, _)| Value::new_string(name))
        .collect();

    Ok(Value::new_array(names))
}

/// `values()` -> liste des valeurs, dans l'ordre de déclaration.
pub fn native_values(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_receiver(args, 1)?;

    let values = args[0]
        .record_fields()?
        .into_iter()
        .map(|(_, value)| value)
        .collect();

    Ok(Value::new_array(values))
}

/// `entries()` -> liste de paires `[nom, valeur]`.
pub fn native_entries(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_receiver(args, 1)?;

    let entries = args[0]
        .record_fields()?
        .into_iter()
        .map(|(name, value)| Value::new_array(vec![Value::new_string(name), value]))
        .collect();

    Ok(Value::new_array(entries))
}

/// `copy()` -> nouveau record (copie superficielle) : `let b = a;` partage le
/// même record, `a.copy()` en fabrique un autre.
pub fn native_copy(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_receiver(args, 1)?;

    Ok(Value::new_record(args[0].record_fields()?))
}

pub fn dispatch_method(name: &str, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    let result = match name {
        "keys" => native_keys(args)?,
        "values" => native_values(args)?,
        "entries" => native_entries(args)?,
        "copy" => native_copy(args)?,
        "to_string" => super::to_string_method(args)?,

        _ => return Ok(None),
    };

    Ok(Some(result))
}
