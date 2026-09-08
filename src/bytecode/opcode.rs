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
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> Self {
        op as u8
    }
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == OpCode::Constant.into() => Ok(OpCode::Constant),
            x if x == OpCode::Nil.into() => Ok(OpCode::Nil),
            x if x == OpCode::True.into() => Ok(OpCode::True),
            x if x == OpCode::False.into() => Ok(OpCode::False),

            x if x == OpCode::Equal.into() => Ok(OpCode::Equal),
            x if x == OpCode::Greater.into() => Ok(OpCode::Greater),
            x if x == OpCode::Less.into() => Ok(OpCode::Less),
            x if x == OpCode::Not.into() => Ok(OpCode::Not),

            x if x == OpCode::Add.into() => Ok(OpCode::Add),
            x if x == OpCode::Subtract.into() => Ok(OpCode::Subtract),
            x if x == OpCode::Multiply.into() => Ok(OpCode::Multiply),
            x if x == OpCode::Divide.into() => Ok(OpCode::Divide),
            x if x == OpCode::Modulo.into() => Ok(OpCode::Modulo),
            x if x == OpCode::Negate.into() => Ok(OpCode::Negate),

            x if x == OpCode::BitAnd.into() => Ok(OpCode::BitAnd),
            x if x == OpCode::BitOr.into() => Ok(OpCode::BitOr),
            x if x == OpCode::BitXor.into() => Ok(OpCode::BitXor),
            x if x == OpCode::BitNot.into() => Ok(OpCode::BitNot),
            x if x == OpCode::ShiftLeft.into() => Ok(OpCode::ShiftLeft),
            x if x == OpCode::ShiftRight.into() => Ok(OpCode::ShiftRight),

            x if x == OpCode::DefineGlobal.into() => Ok(OpCode::DefineGlobal),
            x if x == OpCode::SetGlobal.into() => Ok(OpCode::SetGlobal),
            x if x == OpCode::GetGlobal.into() => Ok(OpCode::GetGlobal),
            x if x == OpCode::GetLocal.into() => Ok(OpCode::GetLocal),
            x if x == OpCode::SetLocal.into() => Ok(OpCode::SetLocal),

            x if x == OpCode::Import.into() => Ok(OpCode::Import),

            x if x == OpCode::JumpIfFalse.into() => Ok(OpCode::JumpIfFalse),
            x if x == OpCode::Jump.into() => Ok(OpCode::Jump),
            x if x == OpCode::Pop.into() => Ok(OpCode::Pop),
            x if x == OpCode::Loop.into() => Ok(OpCode::Loop),
            x if x == OpCode::Call.into() => Ok(OpCode::Call),

            x if x == OpCode::Array.into() => Ok(OpCode::Array),
            x if x == OpCode::GetIndex.into() => Ok(OpCode::GetIndex),
            x if x == OpCode::SetIndex.into() => Ok(OpCode::SetIndex),
            x if x == OpCode::ArrayLength.into() => Ok(OpCode::ArrayLength),
            x if x == OpCode::ArrayPush.into() => Ok(OpCode::ArrayPush),
            x if x == OpCode::ArrayPop.into() => Ok(OpCode::ArrayPop),
            x if x == OpCode::ArrayInsert.into() => Ok(OpCode::ArrayInsert),
            x if x == OpCode::ArrayRemove.into() => Ok(OpCode::ArrayRemove),
            x if x == OpCode::ArrayClear.into() => Ok(OpCode::ArrayClear),
            x if x == OpCode::ArrayContains.into() => Ok(OpCode::ArrayContains),

            x if x == OpCode::Object.into() => Ok(OpCode::Object),

            x if x == OpCode::GetIterator.into() => Ok(OpCode::GetIterator),
            x if x == OpCode::IteratorHasNext.into() => Ok(OpCode::IteratorHasNext),
            x if x == OpCode::IteratorNext.into() => Ok(OpCode::IteratorNext),

            x if x == OpCode::Closure.into() => Ok(OpCode::Closure),
            x if x == OpCode::GetUpvalue.into() => Ok(OpCode::GetUpvalue),
            x if x == OpCode::SetUpvalue.into() => Ok(OpCode::SetUpvalue),

            x if x == OpCode::Return.into() => Ok(OpCode::Return),
            x if x == OpCode::Halt.into() => Ok(OpCode::Halt),

            x if x == OpCode::GetProperty.into() => Ok(OpCode::GetProperty),
            x if x == OpCode::SetProperty.into() => Ok(OpCode::SetProperty),

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
        let max = OpCode::SetProperty as u8;

        for value in 0u8..=u8::MAX {
            if value > max {
                assert!(OpCode::try_from(value).is_err());
            }
        }
    }
}