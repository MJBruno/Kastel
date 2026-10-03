//! Métadonnées sémantiques des sites d'appel.
//!
//! Ce module contient uniquement la représentation du résultat de résolution
//! produit par le TypeChecker et consommé par le compilateur d'émission.
//! Il ne décide ni de l'arité ni de la sélection des signatures : ces règles
//! restent dans `call_resolution`.

use std::collections::HashMap;

use super::{
    builtin_types::{Intrinsic, NativeKind, NativeSpec},
    call_resolution::{ArityError, Callable},
    types::{FunctionType, Type},
};

/// Identifiant stable d'un call-site dans le source Kastel.
///
/// Les expressions d'appel et `new` possèdent déjà une position précise dans
/// l'AST. Le compilateur peut donc réutiliser la décision sémantique sans
/// réanalyser l'expression ni ajouter un identifiant artificiel à l'AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CallSite {
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl CallSite {
    pub(crate) const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// Table des décisions sémantiques produites par le TypeChecker.
///
/// Elle reste indépendante de l'AST et du bytecode : elle associe seulement
/// un call-site source à son `ResolvedCall`.
#[derive(Debug, Clone, Default)]
pub(crate) struct ResolvedCallTable {
    entries: HashMap<CallSite, ResolvedCall>,
}

impl ResolvedCallTable {
    pub(crate) fn insert(&mut self, site: CallSite, resolved: ResolvedCall) {
        self.entries.insert(site, resolved);
    }

    pub(crate) fn get(&self, site: CallSite) -> Option<&ResolvedCall> {
        self.entries.get(&site)
    }

    #[cfg(test)]
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&CallSite, &ResolvedCall)> {
        self.entries.iter()
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

    /// Retourne l'intrinsèque porté par cette résolution, lorsque le call-site
    /// cible effectivement un intrinsèque du langage.
    ///
    /// La connaissance de la représentation `NativeSpec` reste confinée au
    /// noyau de résolution : le compilateur d'émission n'a plus besoin de
    /// connaître `NativeKind` ni de relire le registre natif.
    pub(crate) fn intrinsic(&self) -> Option<Intrinsic> {
        match &self.target {
            CallTarget::Native(spec) => match spec.kind {
                NativeKind::Intrinsic(intrinsic) => Some(intrinsic),
                NativeKind::Runtime => None,
            },
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
    Native(NativeSpec),
    Callable(CallableTarget),
    Constructor {
        class_name: String,
        callable: Option<CallableTarget>,
    },
    Dynamic,
}

