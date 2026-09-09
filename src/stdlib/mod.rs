 
use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

/// Signature commune de toutes les fonctions natives Kastel.
pub type NativeFn = fn(&[Value]) -> Result<Value, RuntimeError>;

pub mod array;
pub mod debug;
pub mod io;
pub mod iterator;
pub mod math;
pub mod dict;
pub mod string;
pub mod system;

/// Enregistre toutes les fonctions natives dans les globals
/// du runtime.
///
/// Cette fonction est appelée une seule fois lors de la création
/// de la VM.
pub fn register_natives(globals: &mut HashMap<String, Value>) {
    io::register(globals);
    math::register(globals);
    string::register(globals);
    array::register(globals);
    dict::register(globals);
    iterator::register(globals);
    system::register(globals);
    debug::register(globals);
}

/// Enregistre tous les noms de fonctions natives connus
/// du compilateur.
///
/// Le compilateur doit connaître les natives avant de compiler
/// le programme principal afin de pouvoir les résoudre comme
/// des variables globales.
pub fn register_compiler_natives(compiler: &mut Compiler) {
    io::register_compiler(compiler);
    math::register_compiler(compiler);
    string::register_compiler(compiler);
    array::register_compiler(compiler);
    dict::register_compiler(compiler);
    iterator::register_compiler(compiler);
    system::register_compiler(compiler);
    debug::register_compiler(compiler);
}

/// Compatibilité avec l'ancien appel.
///
/// Le code existant peut continuer à appeler `execute_native()`.
pub fn execute_native(compiler: &mut Compiler) {
    register_compiler_natives(compiler);
}

