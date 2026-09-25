pub mod capability;
#[allow(clippy::module_inception)]
pub mod compiler;

pub(crate) mod builtin_types;
mod context;
mod control_flow;
mod declarations;
mod emit;
mod expressions;
mod functions;
mod locals;
mod loops;
pub(crate) mod module_types;
mod scope;
mod statements;
pub mod type_checker;
pub mod types;
mod upvalue;
pub(crate) mod variables;
