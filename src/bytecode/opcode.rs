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
    GetLocal,
    SetLocal,

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
}

impl From<OpCode> for u8 {
    #[inline]
    fn from(op: OpCode) -> Self {
        op as u8
    }
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Constant),
            1 => Ok(Self::Nil),
            2 => Ok(Self::True),
            3 => Ok(Self::False),

            4 => Ok(Self::Equal),
            5 => Ok(Self::Greater),
            6 => Ok(Self::Less),
            7 => Ok(Self::Not),
            8 => Ok(Self::Is),

            9 => Ok(Self::Add),
            10 => Ok(Self::Subtract),
            11 => Ok(Self::Multiply),
            12 => Ok(Self::Divide),
            13 => Ok(Self::Modulo),
            14 => Ok(Self::Negate),

            15 => Ok(Self::BitAnd),
            16 => Ok(Self::BitOr),
            17 => Ok(Self::BitXor),
            18 => Ok(Self::BitNot),
            19 => Ok(Self::ShiftLeft),
            20 => Ok(Self::ShiftRight),

            21 => Ok(Self::DefineGlobal),
            22 => Ok(Self::SetGlobal),
            23 => Ok(Self::GetGlobal),
            24 => Ok(Self::GetLocal),
            25 => Ok(Self::SetLocal),

            26 => Ok(Self::Import),

            27 => Ok(Self::JumpIfFalse),
            28 => Ok(Self::Jump),
            29 => Ok(Self::Pop),
            30 => Ok(Self::Loop),
            31 => Ok(Self::Call),

            32 => Ok(Self::Array),
            33 => Ok(Self::GetIndex),
            34 => Ok(Self::SetIndex),
            35 => Ok(Self::ArrayLength),
            36 => Ok(Self::ArrayPush),
            37 => Ok(Self::ArrayPop),
            38 => Ok(Self::ArrayInsert),
            39 => Ok(Self::ArrayRemove),
            40 => Ok(Self::ArrayClear),
            41 => Ok(Self::ArrayContains),

            42 => Ok(Self::Object),

            43 => Ok(Self::GetIterator),
            44 => Ok(Self::IteratorHasNext),
            45 => Ok(Self::IteratorNext),

            46 => Ok(Self::Closure),
            47 => Ok(Self::GetUpvalue),
            48 => Ok(Self::SetUpvalue),

            49 => Ok(Self::Return),
            50 => Ok(Self::Halt),

            51 => Ok(Self::GetProperty),
            52 => Ok(Self::SetProperty),

            53 => Ok(Self::InvokeMethod),
            54 => Ok(Self::InvokeBaseMethod),
            55 => Ok(Self::Class),
            56 => Ok(Self::NewInstance),
            57 => Ok(Self::Interface),

            58 => Ok(Self::PushExceptionHandler),
            59 => Ok(Self::PopExceptionHandler),
            60 => Ok(Self::Throw),
            61 => Ok(Self::FinallyEnd),

            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OpCode;
    use std::convert::TryFrom;

    #[test]
    fn opcode_roundtrip() {
        for value in 0u8..=u8::MAX {
            if let Ok(opcode) = OpCode::try_from(value) {
                assert_eq!(u8::from(opcode), value);
            }
        }
    }

    #[test]
    fn invalid_opcode_is_rejected() {
        let max = OpCode::FinallyEnd as u8;

        for value in 0u8..=u8::MAX {
            if value > max {
                assert!(OpCode::try_from(value).is_err());
            }
        }
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