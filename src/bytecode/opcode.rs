#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Constant,
    Nil,
    True,
    False,

    // COMPARAISON
    Equal,
    Greater,
    Less,
    Not,

    // ARITHMETIQUE
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Negate,

    // BITWISE
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    ShiftLeft,
    ShiftRight,

    // SCOPE
    DefineGlobal,
    SetGlobal,
    GetGlobal,
    GetLocal,
    SetLocal,

    //
    Import,

    // CONTROLE
    JumpIfFalse,
    Jump,

    Pop,
    Loop,
    Call,

    //ARRAY
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

    //OBJECT
    Object,

    //ITERATOR
    GetIterator,
    IteratorHasNext,
    IteratorNext,

    // CLOSURES
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