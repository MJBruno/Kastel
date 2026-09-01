use crate::runtime::value::Value;

// Ré-export : tout le reste du code importe historiquement OpCode via
// `crate::bytecode::chunk::OpCode` (ou `chunk::*`). On garde ce chemin
// valide même si OpCode vit maintenant dans opcode.rs, pour ne pas avoir
// à toucher tous les fichiers qui en dépendent (compiler.rs et sa
// douzaine de sous-modules, machine.rs, etc.).
 pub use super::opcode::OpCode;

/// Stocke le bytecode compilé (instructions + pool de constantes) d'une
/// fonction ou d'un script. La logique d'affichage/désassemblage vit dans
/// `disassembler.rs`, sous forme d'un bloc `impl Chunk` séparé.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// Pousse un octet de bytecode.
    pub fn write(&mut self, byte: u8) {
        self.code.push(byte);
    }

    /// Ajoute une constante dans le pool de constantes.
    /// Retourne l'index de la constante ajoutée, pour faciliter
    /// `OP_CONSTANT <index>`.
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}