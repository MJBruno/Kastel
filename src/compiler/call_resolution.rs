//! Noyau commun de résolution des signatures et contrats d'appel.
//!
//! La résolution statique distingue les signatures fixes, les ensembles de
//! signatures surchargées et les contrats d'arité des natives/intrinsèques.
//! Toutes ces formes passent par la même validation d'arité avant les règles
//! propres à l'appel (instanciation générique, vérification des types, etc.).

use super::{builtin_types::Arity, types::FunctionType};

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
    use crate::compiler::call_metadata::{
        CallSite, CallTarget, CallableTarget, ResolvedCall, ResolvedCallTable,
    };
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
    fn call_target_can_represent_overloaded_user_calls() {
        let signatures = vec![signature(1), signature(2)];
        match CallTarget::Callable(CallableTarget::Overloaded(signatures.clone())) {
            CallTarget::Callable(CallableTarget::Overloaded(found)) => assert_eq!(found.len(), 2),
            _ => panic!("la cible doit conserver l'ensemble des overloads"),
        }
    }

    #[test]
    fn callable_target_selects_unique_and_overloaded_signatures_by_arity() {
        let unique = CallableTarget::One(signature(2));
        assert_eq!(unique.select(2).unwrap().params.len(), 2);
        assert!(unique.select(1).is_err());

        let overloaded = CallableTarget::Overloaded(vec![signature(1), signature(3)]);
        assert_eq!(overloaded.select(3).unwrap().params.len(), 3);
        assert!(overloaded.select(2).is_err());
    }

    #[test]
    fn call_target_can_represent_dynamic_calls() {
        assert!(matches!(CallTarget::Dynamic, CallTarget::Dynamic));
    }

    #[test]
    fn call_target_can_represent_typed_and_native_calls() {
        let signature = signature(1);
        match CallTarget::Callable(CallableTarget::One(signature.clone())) {
            CallTarget::Callable(CallableTarget::One(found)) => {
                assert_eq!(found.params.len(), 1);
            }
            _ => panic!("la cible typée doit conserver la signature"),
        }

        let spec = crate::compiler::builtin_types::spec("print").expect("print est un contrat natif");
        assert!(matches!(CallTarget::Native(spec.clone()), CallTarget::Native(_)));
    }

    #[test]
    fn resolved_call_preserves_selected_signature_and_return_type() {
        let signature = FunctionType {
            generic_params: Vec::new(),
            is_async: false,
            generic_constraints: Vec::new(),
            params: vec![Type::Dynamic],
            return_type: Box::new(Type::Str),
        };
        let resolved = ResolvedCall::from_signature(
            CallTarget::Callable(CallableTarget::One(signature.clone())),
            signature,
        );

        assert!(resolved.signature.is_some());
        assert_eq!(resolved.return_type, Type::Str);
        assert!(matches!(
            resolved.target,
            CallTarget::Callable(CallableTarget::One(_))
        ));
    }

    #[test]
    fn resolved_call_from_callable_preserves_callable_target() {
        let first = signature(1);
        let second = signature(2);
        let target = CallableTarget::Overloaded(vec![first.clone(), second.clone()]);
        let expected_target = target.clone();
        let resolved = ResolvedCall::from_callable(target, second.clone());

        let (target_after, selected, return_type) = resolved.into_parts();
        assert_eq!(selected, Some(second));
        assert_eq!(return_type, Type::None);

        match (target_after, expected_target) {
            (
                CallTarget::Callable(CallableTarget::Overloaded(actual)),
                CallableTarget::Overloaded(expected),
            ) => assert_eq!(actual, expected),
            (actual, expected) => panic!("unexpected targets: {actual:?} vs {expected:?}"),
        }
    }

    #[test]
    fn resolved_call_can_represent_constructor_resolution() {
        let signature = signature(2);
        let callable = CallableTarget::One(signature.clone());
        let resolved = ResolvedCall::from_constructor(
            "Box".to_string(),
            callable.clone(),
            signature.clone(),
            Type::Named("Box".to_string()),
        );

        let (target, selected, return_type) = resolved.into_parts();
        assert_eq!(selected, Some(signature));
        assert_eq!(return_type, Type::Named("Box".to_string()));
        match target {
            CallTarget::Constructor { class_name, callable: Some(found) } => {
                assert_eq!(class_name, "Box");
                assert!(matches!(found, CallableTarget::One(_)));
                assert_eq!(found.select(2).unwrap().params.len(), 2);
            }
            other => panic!("unexpected constructor target: {other:?}"),
        }
    }

    #[test]
    fn resolved_call_can_represent_an_implicit_constructor() {
        let resolved = ResolvedCall::implicit_constructor(
            "Box".to_string(),
            Type::Named("Box".to_string()),
        );

        let (target, selected, return_type) = resolved.into_parts();
        assert!(selected.is_none());
        assert_eq!(return_type, Type::Named("Box".to_string()));
        match target {
            CallTarget::Constructor { class_name, callable: None } => {
                assert_eq!(class_name, "Box");
            }
            other => panic!("unexpected constructor target: {other:?}"),
        }
    }

    #[test]
    fn resolved_call_can_represent_dynamic_resolution() {
        let resolved = ResolvedCall::dynamic();
        assert!(matches!(resolved.target, CallTarget::Dynamic));
        assert!(resolved.signature.is_none());
        assert_eq!(resolved.return_type, Type::Dynamic);
    }

    #[test]
    fn resolved_call_can_represent_special_native_results() {
        let spec = crate::compiler::builtin_types::spec("range")
            .expect("range est un contrat natif")
            .clone();
        let resolved = ResolvedCall::special(CallTarget::Native(spec), Type::Range);

        assert!(resolved.signature.is_none());
        assert_eq!(resolved.return_type, Type::Range);
        assert!(matches!(resolved.target, CallTarget::Native(_)));
    }

    #[test]
    fn resolved_call_exposes_intrinsic_identity() {
        let spawn = crate::compiler::builtin_types::spec("spawn")
            .expect("spawn est un intrinsèque enregistré")
            .clone();
        let resolved = ResolvedCall::special(
            CallTarget::Native(spawn),
            Type::Dynamic,
        );

        assert_eq!(
            resolved.intrinsic(),
            Some(crate::compiler::builtin_types::Intrinsic::Spawn)
        );
    }

    #[test]
    fn resolved_call_does_not_report_runtime_native_as_intrinsic() {
        let println = crate::compiler::builtin_types::spec("println")
            .expect("println est une native runtime")
            .clone();
        let resolved = ResolvedCall::special(
            CallTarget::Native(println),
            Type::None,
        );

        assert_eq!(resolved.intrinsic(), None);
    }

    #[test]
    fn resolved_call_does_not_report_callable_or_dynamic_as_intrinsic() {
        assert_eq!(ResolvedCall::dynamic().intrinsic(), None);

        let signature = signature(1);
        let resolved = ResolvedCall::from_signature(
            CallTarget::Callable(CallableTarget::One(signature.clone())),
            signature,
        );
        assert_eq!(resolved.intrinsic(), None);
    }

    #[test]
    fn resolved_call_does_not_report_constructor_as_intrinsic() {
        let resolved = ResolvedCall::implicit_constructor(
            "Box".to_string(),
            Type::Named("Box".to_string()),
        );

        assert_eq!(resolved.intrinsic(), None);
    }

    #[test]
    fn resolved_call_table_is_keyed_by_source_position() {
        let mut table = ResolvedCallTable::default();
        let site = CallSite::new(7, 12);
        let resolved = ResolvedCall::dynamic();

        table.insert(site, resolved);

        let found = table
            .get(site)
            .expect("le call-site doit être conservé dans la table");
        assert_eq!(found.return_type, Type::Dynamic);
        assert_eq!(table.iter().count(), 1);
        assert!(table.get(CallSite::new(7, 13)).is_none());
    }

    #[test]
    fn distinct_source_positions_can_hold_distinct_resolutions() {
        let mut table = ResolvedCallTable::default();
        table.insert(CallSite::new(1, 1), ResolvedCall::dynamic());
        table.insert(
            CallSite::new(2, 1),
            ResolvedCall::special(
                CallTarget::Native(
                    crate::compiler::builtin_types::spec("range")
                        .expect("range est enregistré")
                        .clone(),
                ),
                Type::Range,
            ),
        );

        assert!(matches!(
            table.get(CallSite::new(1, 1)),
            Some(resolved) if resolved.return_type == Type::Dynamic
        ));
        assert!(matches!(
            table.get(CallSite::new(2, 1)),
            Some(resolved) if resolved.return_type == Type::Range
        ));
        assert_eq!(table.iter().count(), 2);
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
