
// ================================================================
// src/native/operator.rs
// ================================================================

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::value::{NumericOp, Value},
};

use std::collections::HashMap;

pub fn native_add(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    Value::binary_numeric_op(
        args[0].clone(),
        args[1].clone(),
        NumericOp::Add,
    )
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "native_add".to_string(),
        Value::NativeFunction(native_add),
    );
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("native_add");
}
