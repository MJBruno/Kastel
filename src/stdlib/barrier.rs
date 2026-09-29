use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value,
};

pub fn native_barrier(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let parties = match args[0] {
        Value::Integer(value) if value > 0 => {
            usize::try_from(value).map_err(|_| RuntimeError::InvalidFunction)?
        }
        Value::Integer(_) => return Err(RuntimeError::BarrierNonPositive),
        _ => return Err(RuntimeError::TypeError),
    };

    Ok(Value::new_barrier(parties))
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("barrier".into(), Value::NativeFunction(native_barrier));
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("barrier");
}
