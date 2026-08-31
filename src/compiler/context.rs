use std::cell::RefCell;
use std::rc::Rc;

use crate::runtime::upvalue::Upvalue;

use super::locals::LocalTable;

pub type CompilerContextRef = Rc<RefCell<CompilerContext>>;

#[derive(Debug)]
/// Contexte lexical utilisé pendant la compilation d'une fonction ou d'un script.
/// Il contient les variables locales, les upvalues et un lien vers le contexte
/// de compilation de la fonction englobante.
pub struct CompilerContext {
    /// Variables locales appartenant à ce contexte de compilation.
    pub locals: LocalTable,
    /// Variables capturées depuis les contextes englobants.
    pub upvalues: Vec<Upvalue>,
    /// Contexte de compilation de la fonction parente, s'il existe.
    pub enclosing: Option<CompilerContextRef>,
}

impl CompilerContext {
    /// Crée un contexte racine sans fonction englobante.
    pub fn new() -> Self {
        Self {
            locals: LocalTable::new(),
            upvalues: Vec::new(),
            enclosing: None,
        }
    }

    /// Crée un contexte enfant relié au contexte de compilation parent.
    pub fn new_child(enclosing: CompilerContextRef) -> Self {
        Self {
            locals: LocalTable::new(),
            upvalues: Vec::new(),
            enclosing: Some(enclosing),
        }
    }
}