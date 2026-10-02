//! Noyau commun de résolution des signatures d'appel.
//!
//! La résolution statique distingue seulement deux formes de callable :
//! une signature unique et un ensemble de signatures surchargées. L'arité
//! est décidée ici avant l'instanciation générique afin que les fonctions,
//! méthodes et constructeurs utilisent exactement la même règle.

use super::types::FunctionType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArityError {
    pub(crate) expected: i32,
    pub(crate) found: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Callable<'a> {
    One(&'a FunctionType),
    Overloaded(&'a [FunctionType]),
}

impl<'a> Callable<'a> {
    /// Sélectionne la signature d'un appel à partir de l'arité uniquement.
    ///
    /// La validation générique et la compatibilité des types restent dans le
    /// TypeChecker après cette sélection. Ainsi, toutes les formes de
    /// callable passent par exactement le même point de décision.
    pub(crate) fn select(self, arity: usize) -> Result<&'a FunctionType, ArityError> {
        match self {
            Self::One(signature) => {
                if signature.params.len() == arity {
                    Ok(signature)
                } else {
                    Err(ArityError {
                        expected: signature.params.len() as i32,
                        found: arity,
                    })
                }
            }
            Self::Overloaded(signatures) => {
                if let Some(signature) = signatures.iter().find(|signature| signature.params.len() == arity) {
                    return Ok(signature);
                }

                Err(ArityError {
                    expected: signatures
                        .first()
                        .map(|signature| signature.params.len() as i32)
                        .unwrap_or(0),
                    found: arity,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Callable;
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
    fn unique_signature_uses_the_same_arity_rule() {
        let signature = signature(2);
        assert_eq!(Callable::One(&signature).select(2).unwrap().params.len(), 2);
        let error = Callable::One(&signature).select(1).unwrap_err();
        assert_eq!(error.expected, 2);
        assert_eq!(error.found, 1);
    }

    #[test]
    fn overloaded_signature_selects_by_arity() {
        let signatures = [signature(1), signature(3)];
        assert_eq!(Callable::Overloaded(&signatures).select(3).unwrap().params.len(), 3);
        assert!(Callable::Overloaded(&signatures).select(2).is_err());
    }

    #[test]
    fn overloaded_empty_set_reports_zero_expected() {
        let error = Callable::Overloaded(&[]).select(4).unwrap_err();
        assert_eq!(error.expected, 0);
        assert_eq!(error.found, 4);
    }
}
