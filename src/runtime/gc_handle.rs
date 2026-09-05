// ================================================================
// GC HANDLE
// ================================================================
//
// `Gc<T>` est la SEULE façon de faire pointer une `Value` vers quelque
// chose alloué sur le tas et suivi par le collecteur de cycles. Avant
// cette refonte, chaque type (Array, Closure, Module, ...) avait sa
// propre variante `Value::X(Rc<RefCell<X>>)` et son propre registre dans
// gc.rs (arrays, closures, upvalues, objects...). Désormais : un seul
// type de poignée, un seul registre (`Vec<Weak<RefCell<Object>>>`), une
// seule fonction de marquage qui distribue en interne selon la variante
// d'`Object`. Le GC sait exactement ce qu'il possède : tout ce qui est
// `Gc<Object>`, point final.
//
// C'est un newtype, pas une réécriture du mécanisme sous-jacent : le
// comptage de références (Rc) reste le principal outil de gestion
// mémoire, complété par le mark & sweep périodique de gc.rs pour les
// cycles — exactement comme avant, juste unifié derrière un seul type.

use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::rc::{Rc, Weak};

pub struct Gc<T>(Rc<RefCell<T>>);

impl<T> Gc<T> {
    pub fn new(value: T) -> Self {
        Gc(Rc::new(RefCell::new(value)))
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.0.borrow_mut()
    }

    /// Égalité par IDENTITÉ (même allocation), pas par valeur — c'est ce
    /// que le mark & sweep utilise pour savoir si un objet déjà visité a
    /// été rencontré à nouveau (protection contre les cycles pendant le
    /// marquage lui-même).
    pub fn ptr_eq(a: &Gc<T>, b: &Gc<T>) -> bool {
        Rc::ptr_eq(&a.0, &b.0)
    }

    /// Identité sous forme d'entier, utilisée comme clé dans les
    /// `HashSet` de marquage du GC.
    pub fn as_id(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }

    /// Poignée faible, pour le registre du GC (ne maintient PAS l'objet
    /// en vie — exactement ce qu'il faut pour observer "est-ce que cet
    /// objet est encore vivant ?" sans en être la cause).
    pub fn downgrade(&self) -> Weak<RefCell<T>> {
        Rc::downgrade(&self.0)
    }
}

impl<T> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Gc(Rc::clone(&self.0))
    }
}

impl<T: fmt::Debug> fmt::Debug for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Gc({:?})", self.0.borrow())
    }
}

/// Égalité structurelle (compare le CONTENU, pas l'identité) — c'est le
/// comportement attendu pour `==` en Kastel (`[1,2] == [1,2]` doit être
/// vrai même si ce sont deux tableaux distincts en mémoire). Pour une
/// comparaison d'identité, utiliser `Gc::ptr_eq`.
impl<T: PartialEq> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        *self.0.borrow() == *other.0.borrow()
    }
}