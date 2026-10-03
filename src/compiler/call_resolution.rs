//! Noyau commun de résolution des signatures et contrats d'appel.
//!
//! La résolution statique distingue les signatures fixes, les ensembles de
//! signatures surchargées et les contrats d'arité des natives/intrinsèques.
//! Toutes ces formes passent par la même validation d'arité avant les règles
//! propres à l'appel (instanciation générique, vérification des types, etc.).

use super::{
    builtin_types::Arity,
    types::{FunctionType, Type},
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

/// Résultat sémantique minimal d'une résolution d'appel.
///
/// La cible conserve la nature de l'appel, `signature` contient la signature
/// effectivement sélectionnée lorsqu'elle existe, et `return_type` expose le
/// type résultant du call-site. Cette structure reste volontairement légère :
/// elle ne constitue pas encore un HIR complet.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedCall {
    pub(crate) target: CallTarget,
    pub(crate) signature: Option<FunctionType>,
    pub(crate) return_type: Type,
}

impl ResolvedCall {
    pub(crate) fn from_signature(target: CallTarget, signature: FunctionType) -> Self {
        let return_type = *signature.return_type.clone();
        Self {
            target,
            signature: Some(signature),
            return_type,
        }
    }

    pub(crate) fn from_callable(callable: CallableTarget, signature: FunctionType) -> Self {
        Self::from_signature(CallTarget::Callable(callable), signature)
    }

    pub(crate) fn from_constructor(
        class_name: String,
        callable: CallableTarget,
        signature: FunctionType,
        return_type: Type,
    ) -> Self {
        Self {
            target: CallTarget::Constructor {
                class_name,
                callable: Some(callable),
            },
            signature: Some(signature),
            return_type,
        }
    }

    pub(crate) fn implicit_constructor(class_name: String, return_type: Type) -> Self {
        Self {
            target: CallTarget::Constructor {
                class_name,
                callable: None,
            },
            signature: None,
            return_type,
        }
    }

    pub(crate) fn constructor_details(&self) -> Option<(&str, Option<&CallableTarget>)> {
        match &self.target {
            CallTarget::Constructor { class_name, callable } => {
                Some((class_name.as_str(), callable.as_ref()))
            }
            _ => None,
        }
    }

    pub(crate) fn special(target: CallTarget, return_type: Type) -> Self {
        Self {
            target,
            signature: None,
            return_type,
        }
    }

    pub(crate) fn dynamic() -> Self {
        Self::special(CallTarget::Dynamic, Type::Dynamic)
    }

    /// Décompose le résultat de résolution pour le consommateur sémantique.
    ///
    /// Cette API rend explicites les trois informations produites par la
    /// résolution : la cible, la signature éventuellement sélectionnée et le
    /// type de retour. Le `TypeChecker` peut actuellement n'utiliser que ce
    /// dont il a besoin sans rendre les autres champs morts au niveau du
    /// compilateur Rust.
    pub(crate) fn into_parts(self) -> (CallTarget, Option<FunctionType>, Type) {
        (self.target, self.signature, self.return_type)
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

/// Ensemble de signatures utilisateur détenu par une cible d'appel.
///
/// Cette représentation évite de transporter un `Type` complet dans la phase
/// de résolution : une cible utilisateur est soit une signature unique, soit
/// un ensemble d'overloads.
#[derive(Debug, Clone)]
pub(crate) enum CallableTarget {
    One(FunctionType),
    Overloaded(Vec<FunctionType>),
}

impl CallableTarget {
    pub(crate) fn select(&self, arity: usize) -> Result<&FunctionType, ArityError> {
        match self {
            Self::One(signature) => Callable::One(signature).select(arity),
            Self::Overloaded(signatures) => Callable::Overloaded(signatures).select(arity),
        }
    }
}

/// Cible d'un appel après résolution du nom/callee, avant la validation des
/// arguments. Les natives restent des contrats statiques distincts ; les
/// fonctions utilisateur ont une représentation dédiée ; `Dynamic` conserve
/// le fallback dynamique sans réintroduire la représentation `Type` complète.
#[derive(Debug, Clone)]
pub(crate) enum CallTarget {
    Native(super::builtin_types::NativeSpec),
    Callable(CallableTarget),
    Constructor {
        class_name: String,
        callable: Option<CallableTarget>,
    },
    Dynamic,
}

#[cfg(test)]
mod tests {
    use super::{validate_arity, CallTarget, Callable, CallableTarget, ResolvedCall};
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
