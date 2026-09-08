
use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

use std::collections::HashMap;

pub type NativeFn = fn(&[Value]) -> Result<Value, RuntimeError>;

pub mod array;
pub mod clock;
pub mod conversion;
pub mod io;
pub mod iterator;
pub mod operator;
pub mod string;
pub mod types;

/// Enregistre toutes les fonctions natives dans les globals du runtime.
pub fn register_natives(globals: &mut HashMap<String, Value>) {
    clock::register(globals);
    conversion::register(globals);
    types::register(globals);
    iterator::register(globals);
    operator::register(globals);
    io::register(globals);
    string::register(globals);
    array::register(globals);
}

/// Enregistre tous les noms natifs connus du compilateur.
pub fn execute_native(compiler: &mut Compiler) {
    clock::register_compiler(compiler);
    conversion::register_compiler(compiler);
    types::register_compiler(compiler);
    iterator::register_compiler(compiler);
    operator::register_compiler(compiler);
    io::register_compiler(compiler);
    string::register_compiler(compiler);
    array::register_compiler(compiler);
}
 
