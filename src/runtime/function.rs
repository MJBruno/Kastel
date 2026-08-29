// ================================================================
// FUNCTION
// ================================================================

use crate::{bytecode::chunk::Chunk, runtime::upvalue::Upvalue};

 

#[derive(Debug, Clone,PartialEq)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    pub chunk: Chunk,
    pub upvalue_count: usize,
    pub upvalues: Vec<Upvalue>,
}
