use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use crate::runtime::value::Value;

use super::compiler::Compiler;

impl Compiler {
    // ============================================================
    //                      CONSTANTES
    // ============================================================

    /// Ajoute une valeur à la table des constantes et retourne son index sur 8 bits.
    pub(crate) fn make_constant(&mut self, value: Value) -> Result<u8, CompileError> {
        let index = self.chunk.add_constant(value);

        if index > u8::MAX as usize {
            return Err(CompileError::TooManyConstants);
        }

        Ok(index as u8)
    }

    pub(crate) fn identifier_constant(&mut self, name: &str) -> Result<u8, CompileError> {
        self.make_constant(Value::String(name.to_string()))
    }

    // ============================================================
    // BYTECODE
    // ============================================================

    pub(crate) fn emit_byte(&mut self, byte: u8) {
        self.chunk.write(byte);
    }

    pub(crate) fn emit_opcode(&mut self, opcode: OpCode) {
        self.emit_byte(opcode.into());
    }

    pub(crate) fn emit_bytes(&mut self, opcode: OpCode, operand: u8) {
        self.emit_opcode(opcode);
        self.emit_byte(operand);
    }

    pub(crate) fn emit_jump(&mut self, opcode: OpCode) -> usize {
        self.emit_opcode(opcode);
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        self.chunk.code.len() - 2
    }

    pub(crate) fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.code.len() - offset - 2;
        assert!(jump <= u16::MAX as usize, "Jump trop grand");
        let jump = jump as u16;
        self.chunk.code[offset] = (jump >> 8) as u8;
        self.chunk.code[offset + 1] = (jump & 0xff) as u8;
    }

    pub(crate) fn emit_loop(&mut self, loop_start: usize) {
        self.emit_opcode(OpCode::Loop);
        let offset = self.chunk.code.len() + 2 - loop_start;
        assert!(offset <= u16::MAX as usize, "Loop body too large");
        let offset = offset as u16;
        self.emit_byte((offset >> 8) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }
}