//! Infrastructure commune des surcharges par arité.
//!
//! Dans Kastel, une fonction est identifiée pour sa surcharge par le couple
//! `(nom, arité)`. Ce module centralise la règle d'unicité et de sélection par
//! arité afin que le compilateur et le vérificateur de types n'implémentent pas
//! chacun leur propre variante.

use super::types::FunctionType;

/// Permet à une entrée d'un ensemble de surcharges d'exposer son arité.
pub(crate) trait OverloadArity {
    fn overload_arity(&self) -> usize;
}

impl OverloadArity for usize {
    fn overload_arity(&self) -> usize {
        *self
    }
}

impl OverloadArity for FunctionType {
    fn overload_arity(&self) -> usize {
        self.params.len()
    }
}

/// Ensemble ordonné de surcharges d'une même fonction.
///
/// L'ordre d'insertion est conservé pour garder des diagnostics déterministes
/// et préserver le comportement historique lors du choix d'une arité.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverloadSet<T> {
    entries: Vec<T>,
}

impl<T> Default for OverloadSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> OverloadSet<T> {
    pub(crate) fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub(crate) fn from_one(value: T) -> Self {
        Self { entries: vec![value] }
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }


    pub(crate) fn iter(&self) -> std::slice::Iter<'_, T> {
        self.entries.iter()
    }

    pub(crate) fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.entries.iter_mut()
    }

    pub(crate) fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.entries.clone()
    }

    pub(crate) fn into_vec(self) -> Vec<T> {
        self.entries
    }

    /// Ajoute une surcharge uniquement si son arité n'existe pas déjà.
    ///
    /// `Err(arity)` identifie précisément l'arité en collision et permet aux
    /// couches supérieures de produire leur diagnostic spécifique.
    pub(crate) fn insert_unique(&mut self, value: T) -> Result<(), usize>
    where
        T: OverloadArity,
    {
        let arity = value.overload_arity();

        if self.contains_arity(arity) {
            return Err(arity);
        }

        self.entries.push(value);
        Ok(())
    }

    pub(crate) fn contains_arity(&self, arity: usize) -> bool
    where
        T: OverloadArity,
    {
        self.entries
            .iter()
            .any(|entry| entry.overload_arity() == arity)
    }

    // Retourne la surcharge correspondant à une arité précise.
    
    #[allow(dead_code)]
    pub(crate) fn get_by_arity(&self, arity: usize) -> Option<&T>
    where
        T: OverloadArity,
    {
        self.entries
            .iter()
            .find(|entry| entry.overload_arity() == arity)
    }

    /// Retourne mutablement la surcharge correspondant à une arité précise.
    pub(crate) fn get_by_arity_mut(&mut self, arity: usize) -> Option<&mut T>
    where
        T: OverloadArity,
    {
        self.entries
            .iter_mut()
            .find(|entry| entry.overload_arity() == arity)
    }

}

/// Recherche déterministe d'une surcharge par arité.
pub(crate) fn find_by_arity<T: OverloadArity>(entries: &[T], arity: usize) -> Option<&T> {
    entries
        .iter()
        .find(|entry| entry.overload_arity() == arity)
}

impl<'a, T> IntoIterator for &'a OverloadSet<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut OverloadSet<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter_mut()
    }
}

impl<T> IntoIterator for OverloadSet<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{OverloadArity, OverloadSet, find_by_arity};
    use crate::compiler::types::{FunctionType, Type};

    fn signature(arity: usize) -> FunctionType {
        FunctionType {
            generic_params: Vec::new(),
            is_async: false,
            generic_constraints: Vec::new(),
            params: vec![Type::Dynamic; arity],
            return_type: Box::new(Type::None),
        }
    }

    #[test]
    fn duplicate_arities_are_rejected() {
        let mut set = OverloadSet::new();

        assert_eq!(set.insert_unique(signature(1)), Ok(()));
        assert_eq!(set.insert_unique(signature(1)), Err(1));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn overloads_are_selected_by_arity() {
        let mut set = OverloadSet::new();
        set.insert_unique(signature(1)).unwrap();
        set.insert_unique(signature(3)).unwrap();

        assert!(find_by_arity(&set.to_vec(), 1).is_some());
        assert!(find_by_arity(&set.to_vec(), 2).is_none());
        assert_eq!(find_by_arity(&set.to_vec(), 3).unwrap().params.len(), 3);
    }

    #[test]
    fn arity_contract_is_shared_by_function_types_and_plain_arities() {
        let signature = signature(2);
        assert_eq!(signature.overload_arity(), 2);
        assert_eq!(2usize.overload_arity(), 2);
    }
}
