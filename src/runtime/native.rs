use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

#[path = "native/collections.rs"]
mod collections;

#[path = "native/conversion.rs"]
mod conversion;

#[path = "native/io.rs"]
mod io;

#[path = "native/math.rs"]
mod math;

#[path = "native/system.rs"]
mod system;

pub type NativeFn =
    fn(&[Value]) -> Result<Value, RuntimeError>;

pub use collections::{
    native_array_insert,
    native_array_length,
    native_array_pop,
    native_array_push,
    native_array_remove,
    native_list,
};

pub use conversion::{
    native_bool,
    native_float,
    native_int,
    native_str,
    native_type,
};

pub use io::{
    native_format,
    native_input,
    native_print,
    native_println,
};

pub use math::native_add;

pub use system::{
    native_clock,
    native_range,
};

pub fn register_natives(compiler: &mut Compiler) {
    compiler.register_native("clock", native_clock);
    compiler.register_native("int", native_int);
    compiler.register_native("float", native_float);
    compiler.register_native("str", native_str);
    compiler.register_native("bool", native_bool);
    compiler.register_native("type", native_type);
    compiler.register_native("range", native_range);
    compiler.register_native("list", native_list);
    compiler.register_native("native_add", native_add);

    compiler.register_native("println", native_println);
    compiler.register_native("print", native_print);
    compiler.register_native("input", native_input);
    compiler.register_native("format", native_format);

    compiler.register_native("push", native_array_push);
    compiler.register_native("pop", native_array_pop);
    compiler.register_native("length", native_array_length);
    compiler.register_native("insert", native_array_insert);
    compiler.register_native("remove", native_array_remove);
}

pub fn execute_native(
    compiler: &mut Compiler,
    name: &str,
    function: NativeFn,
) -> Result<(), RuntimeError> {
    compiler
        .define_native(name, function)
        .map_err(|_| RuntimeError::NativeError)
}