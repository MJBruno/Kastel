#[allow(clippy::module_inception)]
pub mod compiler;

mod context;
mod control_flow;
mod declarations;
mod emit;
mod expressions;
mod functions;
mod locals;
mod loops;
mod scope;
mod statements;
pub mod type_checker;
pub mod types;
pub(crate) mod builtin_types;
pub(crate) mod module_types;
mod upvalue;
pub(crate) mod variables;
