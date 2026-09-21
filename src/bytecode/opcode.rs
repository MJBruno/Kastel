#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Constant,
    None,
    True,
    False,

    Equal,
    Greater,
    Less,
    Not,
    Is,

    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Negate,

    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    ShiftLeft,
    ShiftRight,

    DefineGlobal,
    SetGlobal,
    GetGlobal,
    SetLocal,
    GetLocal,

    Import,
    ImportAll,

    JumpIfFalse,
    Jump,
    Pop,
    Loop,
    Call,
    Array,
    GetIndex,
    SetIndex,
    ArrayLength,

    Object,

    GetIterator,
    IteratorHasNext,
    IteratorNext,

    Closure,
    GetUpvalue,
    SetUpvalue,

    Return,
    Halt,

    GetProperty,
    SetProperty,

    InvokeMethod,
    InvokeBaseMethod,
    Class,
    NewInstance,
    Interface,

    PushExceptionHandler,
    PopExceptionHandler,
    Throw,
    FinallyEnd,

    // Super-instructions
    AddLocalConst,
    LessLocalConst,
    JumpIfFalsePop,
    LessLocalConstJump,
    LoopLessAddLocalConst,
    AddLocalLocal,
    /// Construit un tuple immuable à partir des N valeurs au sommet
    /// de la pile — ajouté en dernier pour ne décaler aucun
    /// discriminant existant (voir COUNT ci-dessous).
    Tuple,

    /// Préfixe « large » : `Wide <opcode> <haut> <bas> [autres opérandes]`.
    ///
    /// L'opérande CONSTANTE de l'instruction suivante est alors un indice
    /// sur 16 bits (grand-boutiste) au lieu d'un octet, ce qui porte la
    /// limite de 256 à 65 536 constantes par fragment. Seules les
    /// instructions à opérande constante sont concernées : `Constant`,
    /// `DefineGlobal`, `SetGlobal`, `GetGlobal`, `GetProperty`,
    /// `SetProperty`, `InvokeMethod`, `InvokeBaseMethod`, `Closure`,
    /// `Import` et `ImportAll`. Les autres opérandes (nombre d'arguments,
    /// paires d'upvalues...) restent sur un octet.
    ///
    /// Ajouté en dernier pour ne décaler aucun discriminant existant.
    Wide,

    /// Construit un record `{ name: v, ... }` : `Record <n>` retire les N
    /// couples (nom, valeur) de la pile. Ajouté en dernier (aucun
    /// discriminant existant ne bouge).
    Record,
}

impl OpCode {
    /// Nombre total d'opcodes valides.
    pub const COUNT: usize = Self::Record as usize + 1;

    /// Conversion rapide d'un octet de bytecode vers OpCode.
    #[inline(always)]
    pub fn from_byte(value: u8) -> Result<Self, ()> {
        if value as usize >= Self::COUNT {
            return Err(());
        }

        // SAFETY:
        //
        // - OpCode est #[repr(u8)].
        // - Les discriminants commencent à 0.
        // - Ils sont contigus.
        // - Le contrôle précédent garantit que `value` correspond
        //   à un discriminant valide.
        Ok(unsafe { std::mem::transmute::<u8, OpCode>(value) })
    }
}

impl From<OpCode> for u8 {
    #[inline(always)]
    fn from(opcode: OpCode) -> Self {
        opcode as u8
    }
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_byte(value)
    }
}

#[cfg(test)]
mod tests {
    use super::OpCode;
    use std::convert::TryFrom;

    #[test]
    fn opcode_roundtrip() {
        for value in 0..OpCode::COUNT {
            let byte = value as u8;

            let opcode = OpCode::try_from(byte).expect("opcode valide attendu");

            assert_eq!(u8::from(opcode), byte);
        }
    }

    #[test]
    fn from_byte_matches_try_from() {
        for value in 0u8..=u8::MAX {
            assert_eq!(OpCode::from_byte(value), OpCode::try_from(value));
        }
    }

    #[test]
    fn invalid_opcode_is_rejected() {
        for value in OpCode::COUNT..=u8::MAX as usize {
            assert!(OpCode::from_byte(value as u8).is_err());
        }
    }

    #[test]
    fn opcode_count_is_correct() {
        assert_eq!(OpCode::AddLocalConst as u8, 57);
        assert_eq!(OpCode::LessLocalConst as u8, 58);
        assert_eq!(OpCode::JumpIfFalsePop as u8, 59);
        assert_eq!(OpCode::LessLocalConstJump as u8, 60);
        assert_eq!(OpCode::LoopLessAddLocalConst as u8, 61);
        assert_eq!(OpCode::AddLocalLocal as u8, 62);
        assert_eq!(OpCode::Tuple as u8, 63);
        assert_eq!(OpCode::Wide as u8, 64);
        assert_eq!(OpCode::Record as u8, 65);
        assert_eq!(OpCode::COUNT, 66);
    }

    #[test]
    fn exception_opcodes_are_distinct() {
        assert_ne!(OpCode::PushExceptionHandler, OpCode::PopExceptionHandler);
        assert_ne!(OpCode::PopExceptionHandler, OpCode::Throw);
        assert_ne!(OpCode::Throw, OpCode::FinallyEnd);
    }
}
