// ================================================================
// FUNCTION
// ================================================================

use std::rc::Rc;

use crate::{bytecode::chunk::Chunk, runtime::upvalue::Upvalue};

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    /// Bytecode PARTAGÉ : chaque appel crée un `CallFrame` qui n'en prend
    /// qu'une référence (`Rc::clone`). Avant, le fragment entier (code,
    /// tables de positions, constantes) était copié à CHAQUE appel.
    pub chunk: Rc<Chunk>,

    /// Nombre maximal de slots locaux utilisés par cette fonction.
    pub local_count: u16,
    pub upvalue_count: usize,
    pub upvalues: Vec<Upvalue>,
}
