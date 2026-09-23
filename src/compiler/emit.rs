use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use crate::runtime::object::Object;
use crate::runtime::value::Value;

use super::compiler::Compiler;

impl Compiler {
    // ============================================================
    // CONSTANTES
    // ============================================================

    /// Cherche dans le pool du fragment courant une constante identique
    /// (entier, flottant, booléen ou chaîne). Sans cette réutilisation,
    /// chaque occurrence de `println`, de `math`, d'un nom de méthode ou
    /// d'un littéral ajoutait une entrée : un script de quelques dizaines
    /// de lignes dépassait les 256 constantes (opérande sur un octet) et
    /// échouait avec « Trop de constantes dans ce fragment de code ».
    ///
    /// Les fonctions et autres objets ne sont jamais fusionnés.
    fn find_constant(&self, value: &Value) -> Option<u16> {
        let constants = &self.chunk.constants;

        let position = match value {
            Value::Integer(wanted) => constants
                .iter()
                .position(|constant| matches!(constant, Value::Integer(found) if found == wanted)),

            // Comparaison sur les bits : 0.0 et -0.0 restent distincts,
            // et NaN est retrouvé (NaN != NaN sinon).
            Value::Float(wanted) => constants.iter().position(|constant| {
                matches!(constant, Value::Float(found) if found.to_bits() == wanted.to_bits())
            }),

            Value::Boolean(wanted) => constants
                .iter()
                .position(|constant| matches!(constant, Value::Boolean(found) if found == wanted)),

            Value::Object(handle) => {
                let wanted = match &*handle.borrow() {
                    Object::String(text) => text.clone(),
                    _ => return None,
                };

                constants.iter().position(|constant| match constant {
                    Value::Object(other) => {
                        matches!(&*other.borrow(), Object::String(text) if *text == wanted)
                    }
                    _ => false,
                })
            }

            _ => None,
        }?;

        u16::try_from(position).ok()
    }

    /// Ajoute (ou retrouve) une constante et renvoie son indice.
    ///
    /// L'indice tient sur 16 bits : jusqu'à 65 536 constantes par fragment.
    /// Les indices > 255 s'écrivent avec le préfixe `Wide` (voir
    /// `emit_constant_op`).
    pub(crate) fn make_constant(&mut self, value: Value) -> Result<u16, CompileError> {
        if let Some(index) = self.find_constant(&value) {
            return Ok(index);
        }

        let index = self.chunk.constants.len();

        if index > u16::MAX as usize {
            return Err(CompileError::TooManyConstants);
        }

        self.chunk.constants.push(value);

        Ok(index as u16)
    }

    pub(crate) fn identifier_constant(&mut self, name: &str) -> Result<u16, CompileError> {
        self.make_constant(Value::new_string(name.to_string()))
    }

    // ============================================================
    // BYTECODE
    // ============================================================

    pub(crate) fn emit_byte(&mut self, byte: u8) {
        self.chunk
            .write(byte, self.current_line, self.current_column);
    }

    pub(crate) fn emit_opcode(&mut self, opcode: OpCode) {
        self.emit_byte(opcode.into());
    }

    pub(crate) fn emit_bytes(&mut self, opcode: OpCode, operand: u8) {
        self.emit_opcode(opcode);
        self.emit_byte(operand);
    }

    /// Émet `opcode` suivi d'un INDICE DE CONSTANTE.
    ///
    /// - indice <= 255 : `opcode indice` (forme courte, inchangée) ;
    /// - sinon : `Wide opcode haut bas` — le préfixe `Wide` indique à la VM
    ///   que l'opérande constante de l'instruction suivante est sur 2 octets
    ///   (grand-boutiste). Les éventuels autres opérandes (nombre d'arguments
    ///   d'un `InvokeMethod`, paires d'upvalues d'un `Closure`) restent sur
    ///   un octet et sont émis par l'appelant juste après.
    pub(crate) fn emit_constant_op(&mut self, opcode: OpCode, constant: u16) {
        match u8::try_from(constant) {
            Ok(narrow) => self.emit_bytes(opcode, narrow),

            Err(_) => {
                self.emit_opcode(OpCode::Wide);
                self.emit_opcode(opcode);
                self.emit_u16(constant);
            }
        }
    }

    pub(crate) fn emit_u16(&mut self, value: u16) {
        self.emit_byte((value >> 8) as u8);
        self.emit_byte((value & 0xff) as u8);
    }

    pub(crate) fn emit_jump(&mut self, opcode: OpCode) -> usize {
        self.emit_opcode(opcode);
        self.emit_byte(0xff);
        self.emit_byte(0xff);

        self.chunk.code.len() - 2
    }

    pub(crate) fn patch_jump(&mut self, offset: usize) -> Result<(), CompileError> {
        if offset + 1 >= self.chunk.code.len() {
            return Err(CompileError::InvalidJump);
        }

        let jump = self
            .chunk
            .code
            .len()
            .checked_sub(offset)
            .and_then(|value| value.checked_sub(2))
            .ok_or(CompileError::InvalidJump)?;

        if jump > u16::MAX as usize {
            return Err(CompileError::JumpTooLarge);
        }

        let jump = jump as u16;

        self.chunk.code[offset] = (jump >> 8) as u8;
        self.chunk.code[offset + 1] = (jump & 0xff) as u8;

        Ok(())
    }

    pub(crate) fn patch_u16(&mut self, offset: usize, value: usize) -> Result<(), CompileError> {
        if offset + 1 >= self.chunk.code.len() {
            return Err(CompileError::InvalidJump);
        }

        if value > u16::MAX as usize {
            return Err(CompileError::JumpTooLarge);
        }

        let value = value as u16;

        self.chunk.code[offset] = (value >> 8) as u8;
        self.chunk.code[offset + 1] = (value & 0xff) as u8;

        Ok(())
    }

    pub(crate) fn emit_loop(&mut self, loop_start: usize) -> Result<(), CompileError> {
        if loop_start > self.chunk.code.len() {
            return Err(CompileError::InvalidJump);
        }

        self.emit_opcode(OpCode::Loop);

        let offset = self
            .chunk
            .code
            .len()
            .checked_add(2)
            .and_then(|value| value.checked_sub(loop_start))
            .ok_or(CompileError::InvalidJump)?;

        if offset > u16::MAX as usize {
            return Err(CompileError::JumpTooLarge);
        }

        let offset = offset as u16;

        self.emit_byte((offset >> 8) as u8);
        self.emit_byte((offset & 0xff) as u8);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler::compiler::Compiler;
    use crate::frontend::{lexer::lexer::Lexer, parser::Parser};
    use crate::runtime::value::Value;
    use crate::stdlib::execute_native;

    fn compile(source: &str) -> Result<(), crate::error::compile_error::CompileError> {
        let tokens = Lexer::new(source.to_string()).scan_token().unwrap();
        let statements = Parser::new(tokens).parse().unwrap();

        let mut compiler = Compiler::new();
        execute_native(&mut compiler);
        compiler.compile(&statements).map(|_| ())
    }

    #[test]
    fn repeated_names_and_literals_share_one_constant() {
        // 400 appels : sans dédoublonnage, `println` et `1` occuperaient
        // 800 entrées et dépasseraient la limite de 256 constantes.
        let source = "println(1);\n".repeat(400);
        let result = compile(&source);

        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn more_than_256_distinct_constants_are_supported() {
        let source = (0..300)
            .map(|index| format!("println(\"s{index}\");\n"))
            .collect::<String>();

        assert!(compile(&source).is_ok());
    }

    /// Exécute vraiment un programme de plus de 256 constantes : lectures et
    /// écritures de globales, propriétés, appels de méthode et fonction
    /// déclarée APRÈS le dépassement (opérandes `Wide`).
    #[test]
    fn programs_with_more_than_256_constants_run_correctly() {
        let mut source = String::from("let total = 0;\n");

        for index in 0..300 {
            source.push_str(&format!("total = total + {};\n", 1000 + index));
        }

        source.push_str(
            r#"
let a = [1];
a.add(2);
let n = a.size();
let o = {k: 5};
o.k = o.k + 1;
let k = o.k;
func plus_one(x) {
    return x + 1;
}
let r = plus_one(total);
"#,
        );

        let tokens = Lexer::new(source).scan_token().unwrap();
        let statements = Parser::new(tokens).parse().unwrap();

        let mut compiler = Compiler::new();
        execute_native(&mut compiler);

        let function = std::rc::Rc::new(compiler.compile(&statements).unwrap());
        assert!(function.chunk.constants.len() > 256);

        let mut vm = crate::vm::machine::VirtualMachine::new(function, None);
        vm.run().unwrap();

        let globals = vm.globals.borrow();

        // 300 * 1000 + (0 + 1 + ... + 299)
        assert!(matches!(
            globals.get("total"),
            Some(Value::Integer(344_850))
        ));
        assert!(matches!(globals.get("n"), Some(Value::Integer(2))));
        assert!(matches!(globals.get("k"), Some(Value::Integer(6))));
        assert!(matches!(globals.get("r"), Some(Value::Integer(344_851))));
    }
}
