//! Interaction avec le système/processus, native.
//!
//! Volontairement limité à des opérations de LECTURE (nom de l'OS,
//! arguments, sortie propre) : pas d'exécution de commande shell
//! arbitraire (`os.system(...)`) — c'est une surface d'attaque
//! (injection de commande) largement plus sensible que le reste de
//! ce lot, et rien dans la demande n'en avait besoin. À ajouter
//! séparément, en connaissance de cause, si un vrai besoin se
//! présente.

use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value,
};

pub fn native_os_name(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    Ok(Value::new_string(std::env::consts::OS.to_string()))
}

pub fn native_os_arch(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    Ok(Value::new_string(std::env::consts::ARCH.to_string()))
}

/// Arguments de la ligne de commande (y compris l'exécutable lui-même
/// en position 0, comme `sys.argv` en Python).
pub fn native_args(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    let values = std::env::args().map(Value::new_string).collect::<Vec<_>>();

    Ok(Value::new_array(values))
}

/// Termine immédiatement le processus avec le code de sortie donné
/// (0 si omis). Ne revient jamais — tout code Kastel après cet appel
/// n'est jamais exécuté, comme `sys.exit()`/`process.exit()`.
pub fn native_exit(args: &[Value]) -> Result<Value, RuntimeError> {
    let code = match args.len() {
        0 => 0,
        1 => match &args[0] {
            Value::Integer(code) => *code as i32,
            _ => return Err(RuntimeError::TypeError),
        },
        found => {
            return Err(RuntimeError::WrongArgumentCount { expected: 1, found });
        }
    };

    std::process::exit(code);
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
    register_one(globals, "os_name", native_os_name);
    register_one(globals, "os_arch", native_os_arch);
    register_one(globals, "args", native_args);
    register_one(globals, "exit", native_exit);
}

pub fn register_compiler(compiler: &mut Compiler) {
    define_one(compiler, "os_name");
    define_one(compiler, "os_arch");
    define_one(compiler, "args");
    define_one(compiler, "exit");
}
