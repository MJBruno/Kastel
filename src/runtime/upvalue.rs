#[derive(Debug, Clone, PartialEq)]
/// Décrit une variable capturée par une closure.
/// `index` désigne soit un slot local, soit un index d'upvalue du contexte parent.
/// `is_local` indique lequel des deux cas s'applique.",
pub struct Upvalue {
    /// Index du slot local ou de l'upvalue dans le contexte source.",
    pub index: u8,
    /// Vrai lorsque l'upvalue capture directement une variable locale du parent.",
    pub is_local: bool,
}