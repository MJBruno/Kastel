use crate::runtime::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Upvalue {
    /// Index du slot local ou de l'upvalue dans le contexte source.
    pub index: u8,

    /// Vrai lorsque l'upvalue capture directement une variable locale du parent.
    pub is_local: bool,
}


#[allow(dead_code)]
impl Upvalue {
    pub fn new(index: u8, is_local: bool) -> Self {
        Self { index, is_local }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjUpvalue {
    /// Position absolue dans la pile tant que l'upvalue est ouverte.
    pub slot: usize,

    /// Valeur conservée lorsque l'upvalue est fermée.
    pub closed: Option<Value>,
}

impl ObjUpvalue {
    pub fn new(slot: usize) -> Self {
        Self {
            slot,
            closed: None,
        }
    }
}