use std::convert::TryFrom;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Constant,
    Nil,
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

    JumpIfFalse,
    Jump,
    Pop,
    Loop,
    Call,
    Array,
    GetIndex,
    SetIndex,
    ArrayLength,
    ArrayPush,
    ArrayPop,
    ArrayInsert,
    ArrayRemove,
    ArrayClear,
    ArrayContains,

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
    SetLocalPop,
    AddLocalConst,
    LessLocalConst,
    JumpIfFalsePop,
}

impl OpCode {
    /// Nombre total d'opcodes valides.
    pub const COUNT: usize = Self::JumpIfFalsePop as usize + 1;

    /// Conversion rapide d'un octet de bytecode vers OpCode.
    ///
    /// Cette fonction est utilisée directement par le dispatcher VM
    /// afin d'éviter le décodage par `match` de `TryFrom<u8>`.
    #[inline(always)]
    pub fn from_byte(value: u8) -> Result<Self, ()> {
        if value as usize >= Self::COUNT {
            return Err(());
        }

        // SAFETY:
        //
        // - OpCode est #[repr(u8)].
        // - Les discriminants sont implicites.
        // - Ils commencent à 0.
        // - Ils sont contigus jusqu'à JumpIfFalsePop.
        // - Le contrôle précédent garantit que `value` correspond
        //   à un discriminant valide.
        Ok(unsafe {
            std::mem::transmute::<u8, OpCode>(value)
        })
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

            let opcode =
                OpCode::try_from(byte)
                    .expect("opcode valide attendu");

            assert_eq!(u8::from(opcode), byte);
        }
    }

    #[test]
    fn from_byte_matches_try_from() {
        for value in 0u8..=u8::MAX {
            assert_eq!(
                OpCode::from_byte(value),
                OpCode::try_from(value)
            );
        }
    }

    #[test]
    fn invalid_opcode_is_rejected() {
        for value in OpCode::COUNT..=u8::MAX as usize {
            assert!(
                OpCode::from_byte(value as u8).is_err()
            );
        }
    }

    #[test]
    fn opcode_count_is_correct() {
        assert_eq!(OpCode::SetLocalPop as u8, 62);
        assert_eq!(OpCode::AddLocalConst as u8, 63);
        assert_eq!(OpCode::LessLocalConst as u8, 64);
        assert_eq!(OpCode::JumpIfFalsePop as u8, 65);
        assert_eq!(OpCode::COUNT, 66);
    }

    #[test]
    fn exception_opcodes_are_distinct() {
        assert_ne!(
            OpCode::PushExceptionHandler,
            OpCode::PopExceptionHandler
        );

        assert_ne!(
            OpCode::Throw,
            OpCode::FinallyEnd
        );

        assert_ne!(
            OpCode::PushExceptionHandler,
            OpCode::Throw
        );
    }
}