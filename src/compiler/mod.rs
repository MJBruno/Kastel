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
mod upvalue;
mod variables;