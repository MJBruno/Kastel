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
///
/// `lines`/`columns` sont des tableaux PARALLÈLES à `code` : `lines[i]`/
/// `columns[i]` donnent la position source qui a produit l'octet `code[i]`.
/// C'est ce qui permet à la VM de reporter une position précise pour une
/// erreur runtime — il suffit de regarder `chunk.position_at(ip)` au
/// moment de l'erreur, sans avoir à faire transiter la moindre info de
/// position à travers Value/RuntimeError. Même principe que clox
/// (Crafting Interpreters, ch. 14), étendu avec la colonne.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: Vec<usize>,
    pub columns: Vec<usize>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            lines: Vec::new(),
            columns: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// Pousse un octet de bytecode, avec la position source qui l'a produit.
    pub fn write(&mut self, byte: u8, line: usize, column: usize) {
        self.code.push(byte);
        self.lines.push(line);
        self.columns.push(column);
    }

    /// Position (ligne, colonne) correspondant à un offset de bytecode
    /// donné. Retourne (0, 0) ("inconnue") si l'offset est hors limites
    /// plutôt que de paniquer — un offset invalide ne doit jamais faire
    /// planter l'affichage d'une erreur, ce serait perdre le message
    /// d'erreur d'origine pour un problème d'affichage secondaire.
    pub fn position_at(&self, offset: usize) -> (usize, usize) {
        let line = self.lines.get(offset).copied().unwrap_or(0);
        let column = self.columns.get(offset).copied().unwrap_or(0);

        (line, column)
    }

    /// Ajoute une constante dans le pool de constantes.
    /// Retourne l'index de la constante ajoutée, pour faciliter
    /// `OP_CONSTANT <index>`.
    #[allow(dead_code)]
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}
