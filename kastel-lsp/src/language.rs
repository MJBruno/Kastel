//! Constantes du langage Kastel partagées entre LSP et analyse.

pub const KEYWORDS: &[&str] = &[
    "const",
    "let",
    "func",
    "class",
    "this",
    "interface",
    "if",
    "else",
    "while",
    "for",
    "in",
    "return",
    "break",
    "continue",
    "import",
    "from",
    "export",
    "try",
    "catch",
    "finally",
    "true",
    "false",
    "None",
];

/// Fonctions/valeurs fournies par le runtime Kastel.
/// À étendre au fur et à mesure que la stdlib grossit.
pub const BUILTINS: &[&str] = &[
    "print", "println", "len", "push", "pop", "str", "int", "float", "bool", "char", "Some", "Ok",
    "Err", "panic",
];
