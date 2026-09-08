use crate::{
    error::runtime_error::RuntimeError,
    runtime::value::{NumericOp, Value},
};

pub fn native_add(
    args: &[Value],
) -> Result<Value, RuntimeError> {
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