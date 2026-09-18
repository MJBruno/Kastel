use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::runtime::{function::Function, gc_handle::Gc, object::Object, upvalue::ObjUpvalue, value::Value};

#[derive(Debug, Clone)]
pub struct Closure {
    pub function: Rc<Function>,
    pub upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,

    /// Environnement global du module dans lequel cette closure a été créée.
    ///
    /// Une référence faible évite de créer un cycle : l'environnement est
    /// détenu par la VM ou par `ModuleInstance`, tandis que la closure ne
    /// fait que le référencer.
    pub global_env: Weak<RefCell<HashMap<String, Value>>>,

    /// Classe qui possède cette méthode.
    ///
    /// `None` pour les closures ordinaires.
    pub owner_class: Option<Gc<Object>>,
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        let same_owner_class = match (&self.owner_class, &other.owner_class) {
            (None, None) => true,
            (Some(left), Some(right)) => Gc::ptr_eq(left, right),
            _ => false,
        };

        // `global_env` n'entre pas dans l'égalité d'une closure : il s'agit
        // du contexte d'exécution lexical du module, pas de son identité
        // fonctionnelle observable.
        self.function == other.function
            && self.upvalues == other.upvalues
            && same_owner_class
    }
}
