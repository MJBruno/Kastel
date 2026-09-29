use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

/// Construit un canal coopératif. Sans argument, le canal est non borné.
/// Avec un entier strictement positif, il est borné à cette capacité.
pub fn native_channel(args: &[Value]) -> Result<Value, RuntimeError> {
    match args {
        [] => Ok(Value::new_channel()),

        [Value::Integer(capacity)] if *capacity > 0 => {
            Ok(Value::new_bounded_channel(*capacity as usize))
        }

        [Value::Integer(_)] => Err(RuntimeError::ChannelNonPositive),

        [_] => Err(RuntimeError::TypeError),

        _ => Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        }),
    }
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("channel".into(), Value::NativeFunction(native_channel));
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("channel");
}
