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
<<<<<<< HEAD
=======
    ImportAll,

>>>>>>> 6a6d144 (Stabilisation de kastel)
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
    LessLocalConstJump,
    LoopLessAddLocalConst,
    AddLocalLocal,
}

impl OpCode {
    /// Nombre total d'opcodes valides.
    pub const COUNT: usize = Self::AddLocalLocal as usize + 1;
    /// Conversion rapide d'un octet de bytecode vers OpCode.
    ///
    /// Cette fonction est utilisée directement par le dispatcher VM.
    #[inline(always)]
    pub fn from_byte(value: u8) -> Result<Self, ()> {
        if value as usize >= Self::COUNT {
            return Err(());
        }

        // SAFETY:
        //
        // - OpCode est #[repr(u8)].
        // - Les discriminants commencent à 0.
        // - Ils sont contigus jusqu'à LessLocalConstJump.
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
<<<<<<< HEAD
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
        assert_eq!(OpCode::SetLocalPop as u8, 62);
        assert_eq!(OpCode::AddLocalConst as u8, 63);
        assert_eq!(OpCode::LessLocalConst as u8, 64);
        assert_eq!(OpCode::JumpIfFalsePop as u8, 65);
        assert_eq!(OpCode::LessLocalConstJump as u8, 66);
        assert_eq!(OpCode::LoopLessAddLocalConst as u8, 67);
        assert_eq!(OpCode::AddLocalLocal as u8, 68);
        assert_eq!(OpCode::COUNT, 69);
    }

    #[test]
    fn exception_opcodes_are_distinct() {
        assert_ne!(OpCode::PushExceptionHandler, OpCode::PopExceptionHandler);
        assert_ne!(OpCode::PopExceptionHandler, OpCode::Throw);
        assert_ne!(OpCode::Throw, OpCode::FinallyEnd);
    }
}
=======
        match value {
            x if x == OpCode::Constant.into() => Ok(OpCode::Constant),
            x if x == OpCode::Nil.into() => Ok(OpCode::Nil),
            x if x == OpCode::True.into() => Ok(OpCode::True),
            x if x == OpCode::False.into() => Ok(OpCode::False),

            x if x == OpCode::Equal.into() => Ok(OpCode::Equal),
            x if x == OpCode::Greater.into() => Ok(OpCode::Greater),
            x if x == OpCode::Less.into() => Ok(OpCode::Less),
            x if x == OpCode::Not.into() => Ok(OpCode::Not),
            x if x == OpCode::Is.into() => Ok(OpCode::Is),

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
            x if x == OpCode::ImportAll.into() => Ok(OpCode::ImportAll),

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

            x if x == OpCode::InvokeMethod.into() => Ok(OpCode::InvokeMethod),
            x if x == OpCode::InvokeBaseMethod.into() => Ok(OpCode::InvokeBaseMethod),
            x if x == OpCode::Class.into() => Ok(OpCode::Class),
            x if x == OpCode::NewInstance.into() => Ok(OpCode::NewInstance),
            x if x == OpCode::Interface.into() => Ok(OpCode::Interface),

            x if x == OpCode::PushExceptionHandler.into() => {
                Ok(OpCode::PushExceptionHandler)
            }

            x if x == OpCode::PopExceptionHandler.into() => {
                Ok(OpCode::PopExceptionHandler)
            }

            x if x == OpCode::Throw.into() => Ok(OpCode::Throw),
            x if x == OpCode::FinallyEnd.into() => Ok(OpCode::FinallyEnd),

            _ => Err(()),
        }
    }
}
>>>>>>> 6a6d144 (Stabilisation de kastel)
