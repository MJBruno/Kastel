use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value,
};

/// Signature commune de toutes les fonctions natives Kastel.
pub type NativeFn = fn(&[Value]) -> Result<Value, RuntimeError>;

pub mod array;
pub mod debug;
pub mod dict;
pub mod file;
pub mod io;
pub mod iterator;
pub mod json;
pub mod math;
pub mod os;
pub mod path;
pub mod string;
pub mod system;
pub mod tuple;

/// Enregistre toutes les fonctions natives dans les globals du runtime.
pub fn register_natives(globals: &mut HashMap<String, Value>) {
    io::register(globals);
    math::register(globals);
    string::register(globals);
    array::register(globals);
    tuple::register(globals);
    dict::register(globals);
    // object::register(globals);
    iterator::register(globals);
    system::register(globals);
    debug::register(globals);
    json::register(globals);
    file::register(globals);
    path::register(globals);
    os::register(globals);
}

/// Enregistre les natives connues du compilateur.
pub fn register_compiler_natives(compiler: &mut Compiler) {
    io::register_compiler(compiler);
    math::register_compiler(compiler);
    string::register_compiler(compiler);
    array::register_compiler(compiler);
    tuple::register_compiler(compiler);
    dict::register_compiler(compiler);
    // object::register_compiler(compiler);
    iterator::register_compiler(compiler);
    system::register_compiler(compiler);
    debug::register_compiler(compiler);
    json::register_compiler(compiler);
    file::register_compiler(compiler);
    path::register_compiler(compiler);
    os::register_compiler(compiler);
}

/// Compatibilité avec l'ancien appel.
pub fn execute_native(compiler: &mut Compiler) {
    register_compiler_natives(compiler);
}
