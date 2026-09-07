// ================================================================
// FUNCTION
// ================================================================

use crate::{bytecode::chunk::Chunk, runtime::upvalue::Upvalue};

 

#[derive(Debug, Clone,PartialEq)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    pub chunk: Chunk,

     /// Nombre maximal de slots locaux utilisés par cette fonction.
    pub local_count: u8,
    pub upvalue_count: usize,
    pub upvalues: Vec<Upvalue>,
}
