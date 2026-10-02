//! Noyau commun de résolution des signatures et contrats d'appel.
//!
//! La résolution statique distingue les signatures fixes, les ensembles de
//! signatures surchargées et les contrats d'arité des natives/intrinsèques.
//! Toutes ces formes passent par la même validation d'arité avant les règles
//! propres à l'appel (instanciation générique, vérification des types, etc.).

use super::{
    builtin_types::Arity,
    types::FunctionType,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArityError {
    pub(crate) expected: i32,
    pub(crate) found: usize,
}

/// Contrat d'arité minimal commun aux différents callable du compilateur.
///
/// Les signatures de fonctions ont une arité exacte, tandis que les natives
/// peuvent accepter une plage ou un nombre minimal d'arguments.
pub(crate) trait CallArity {
    fn accepts_arity(&self, found: usize) -> bool;
    fn expected_arity(&self, found: usize) -> usize;
}

impl CallArity for FunctionType {
    fn accepts_arity(&self, found: usize) -> bool {
        self.params.len() == found
    }

    fn expected_arity(&self, _found: usize) -> usize {
        self.params.len()
    }
}

impl CallArity for Arity {
    fn accepts_arity(&self, found: usize) -> bool {
        self.accepts(found)
    }

    fn expected_arity(&self, found: usize) -> usize {
        self.expected_for(found)
    }
}

/// Valide un contrat d'arité sans imposer de représentation particulière du
/// callable. C'est le point commun entre signatures fixes et natives.
pub(crate) fn validate_arity<T: CallArity>(contract: &T, found: usize) -> Result<(), ArityError> {
    if contract.accepts_arity(found) {
        Ok(())
    } else {
        Err(ArityError {
            expected: contract.expected_arity(found) as i32,
            found,
        })
    }
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
                validate_arity(signature, arity)?;
                Ok(signature)
            }
            Self::Overloaded(signatures) => {
                if let Some(signature) = signatures.iter().find(|signature| {
                    signature.params.len() == arity
                }) {
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
    use super::{validate_arity, Callable};
    use crate::compiler::builtin_types::Arity;
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

    #[test]
    fn native_exact_arity_uses_the_shared_validator() {
        assert!(validate_arity(&Arity::Exact(2), 2).is_ok());
        let error = validate_arity(&Arity::Exact(2), 1).unwrap_err();
        assert_eq!(error.expected, 2);
        assert_eq!(error.found, 1);
    }

    #[test]
    fn native_range_and_at_least_arity_use_the_shared_validator() {
        assert!(validate_arity(&Arity::Range { min: 1, max: 3 }, 2).is_ok());
        assert!(validate_arity(&Arity::AtLeast(1), 4).is_ok());

        let range_error = validate_arity(&Arity::Range { min: 1, max: 3 }, 5).unwrap_err();
        assert_eq!(range_error.expected, 3);
        assert_eq!(range_error.found, 5);

        let at_least_error = validate_arity(&Arity::AtLeast(2), 1).unwrap_err();
        assert_eq!(at_least_error.expected, 2);
        assert_eq!(at_least_error.found, 1);
    }
}
