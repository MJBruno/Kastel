//! Sources de vérité UNIQUES de la bibliothèque native.
//!
//! Avant, ajouter une fonction native ou une méthode de collection obligeait
//! à modifier de mémoire 3 endroits qui dérivaient : l'implémentation +
//! l'enregistrement runtime, l'enregistrement compilateur, et les tables de
//! types du vérificateur (`rand_range` typé à 1 argument alors que la native
//! en prend 2, par exemple). Désormais chaque élément est déclaré UNE SEULE
//! fois, dans une table :
//!
//! * **fonctions globales** : `NATIVES` de chaque module (`math::NATIVES`…).
//!   Le runtime, le compilateur (`define_native`) et le vérificateur de types
//!   (`builtin_types`) en sont tous DÉRIVÉS.
//! * **méthodes de collections** : `METHODS` de chaque type (`array::METHODS`…).
//!   La répartition à l'exécution (`dispatch_method`) et le typage statique
//!   (`Type::method_signature`) en sont tous deux dérivés.
//!
//! Les tests de ce module vérifient la cohérence (noms uniques, arités
//! déclarées = arités attendues par les fonctions).

use crate::error::runtime_error::RuntimeError;
use crate::runtime::value::Value;

use super::{NativeFn, renamed_method_error};

// ============================================================
//                    FONCTIONS GLOBALES
// ============================================================

/// Types utilisables dans la déclaration d'une fonction native. Le
/// vérificateur de types les traduit en `Type` (voir `builtin_types`) ; ce
/// module reste ainsi indépendant du compilateur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeTag {
    Dynamic,
    Int,
    Float,
    Bool,
    Str,
    /// `None` (absence de valeur).
    Unit,
    ArrayDynamic,
    ArrayOfStr,
    DictOfDynamic,
}

#[derive(Debug, Clone, Copy)]
pub enum Signature {
    /// Arité variable ou optionnelle : le vérificateur ne contrôle rien
    /// (`println`, `range`, `format`…).
    Untyped,

    Function {
        params: &'static [TypeTag],
        returns: TypeTag,
    },
}

#[derive(Clone, Copy)]
pub struct NativeSpec {
    pub name: &'static str,
    pub function: NativeFn,
    pub signature: Signature,
}

/// Native à signature fixe.
pub const fn native(
    name: &'static str,
    function: NativeFn,
    params: &'static [TypeTag],
    returns: TypeTag,
) -> NativeSpec {
    NativeSpec {
        name,
        function,
        signature: Signature::Function { params, returns },
    }
}

/// Native d'arité variable ou optionnelle (non typée).
pub const fn untyped_native(name: &'static str, function: NativeFn) -> NativeSpec {
    NativeSpec {
        name,
        function,
        signature: Signature::Untyped,
    }
}

// ============================================================
//                  MÉTHODES DE COLLECTIONS
// ============================================================

/// Type d'un paramètre de méthode, relatif au receveur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamTag {
    /// N'importe quelle valeur (`dynamic`).
    Any,

    /// Le type des éléments du receveur (`Array<int>.add` attend un `int`).
    Element,
}

/// Type du résultat d'une méthode, relatif au receveur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnTag {
    Int,
    Bool,
    Str,
    Unit,
    Dynamic,

    /// Le type du receveur lui-même (`copy`, `union`…).
    SelfType,

    /// Le type des éléments (des valeurs, pour un dict).
    Element,

    ArrayOfElement,
    ArrayOfKeys,
    ArrayOfValues,

    /// Tableau de paires `[clé, valeur]` (`Dict.entries`).
    ArrayOfPairs,
}

#[derive(Debug, Clone, Copy)]
pub enum MethodSig {
    /// La méthode existe mais n'est pas typée : le résultat reste dynamique.
    Untyped,

    Typed {
        params: &'static [ParamTag],
        returns: ReturnTag,
    },
}

#[derive(Clone, Copy)]
pub struct MethodSpec {
    pub name: &'static str,

    /// `None` : méthode exécutée directement par la VM (elle rappelle du
    /// code Kastel : `map`, `filter`… ; ou `iter`). `dispatch` renvoie alors
    /// `Ok(None)` et l'appelant prend le relais.
    pub function: Option<NativeFn>,

    pub signature: MethodSig,
}

/// Méthode typée et implémentée par une fonction native.
pub const fn method(
    name: &'static str,
    function: NativeFn,
    params: &'static [ParamTag],
    returns: ReturnTag,
) -> MethodSpec {
    MethodSpec {
        name,
        function: Some(function),
        signature: MethodSig::Typed { params, returns },
    }
}

/// Méthode existante mais non typée.
pub const fn untyped_method(name: &'static str, function: NativeFn) -> MethodSpec {
    MethodSpec {
        name,
        function: Some(function),
        signature: MethodSig::Untyped,
    }
}

/// Méthode typée exécutée par la VM (pas de fonction native).
pub const fn vm_method(
    name: &'static str,
    params: &'static [ParamTag],
    returns: ReturnTag,
) -> MethodSpec {
    MethodSpec {
        name,
        function: None,
        signature: MethodSig::Typed { params, returns },
    }
}

/// Méthode non typée exécutée par la VM.
pub const fn untyped_vm_method(name: &'static str) -> MethodSpec {
    MethodSpec {
        name,
        function: None,
        signature: MethodSig::Untyped,
    }
}

/// Nom supprimé au profit d'un remplaçant (`length` -> `size()`).
#[derive(Debug, Clone, Copy)]
pub struct RenamedMethod {
    pub old: &'static str,
    pub replacement: &'static str,

    /// Le nom peut aussi être une CLÉ de dict (`d.length`) : le
    /// vérificateur de types ne le signale alors pas sur un accès sans
    /// parenthèses. À l'exécution, l'APPEL `d.length()` échoue toujours avec
    /// l'indice.
    pub may_be_key: bool,
}

pub const fn renamed(old: &'static str, replacement: &'static str) -> RenamedMethod {
    RenamedMethod {
        old,
        replacement,
        may_be_key: false,
    }
}

pub const fn renamed_maybe_key(old: &'static str, replacement: &'static str) -> RenamedMethod {
    RenamedMethod {
        old,
        replacement,
        may_be_key: true,
    }
}

/// Noms supprimés pour TOUTES les collections.
pub const COMMON_RENAMED: &[RenamedMethod] = &[renamed("to_iterator", "iter()")];

/// Table des méthodes d'un type de collection.
pub struct MethodTable {
    pub methods: &'static [MethodSpec],
    pub renamed: &'static [RenamedMethod],

    /// `true` : une méthode absente de la table est une erreur certaine
    /// (`Set`, dont l'API est fermée). `false` : le type a d'autres
    /// méthodes non déclarées ici (chaînes, tableaux…) et un nom inconnu
    /// reste simplement dynamique.
    pub strict: bool,
}

impl MethodTable {
    pub fn find(&self, name: &str) -> Option<&MethodSpec> {
        self.methods.iter().find(|spec| spec.name == name)
    }

    /// Remplaçant d'un nom supprimé. `for_static` : ne renvoie pas les noms
    /// qui peuvent être des clés de dict (voir `RenamedMethod::may_be_key`).
    pub fn renamed_replacement(&self, name: &str, for_static: bool) -> Option<&'static str> {
        self.renamed
            .iter()
            .chain(COMMON_RENAMED.iter())
            .find(|entry| entry.old == name && !(for_static && entry.may_be_key))
            .map(|entry| entry.replacement)
    }

    /// Répartition à l'exécution : `args[0]` est le receveur.
    ///
    /// * nom supprimé -> erreur guidée ;
    /// * méthode native -> son résultat ;
    /// * méthode VM ou inconnue -> `Ok(None)` (l'appelant décide).
    pub fn dispatch(&self, name: &str, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
        if let Some(replacement) = self.renamed_replacement(name, false) {
            return Err(renamed_method_error(name, replacement));
        }

        match self.find(name).and_then(|spec| spec.function) {
            Some(function) => Ok(Some(function(args)?)),
            None => Ok(None),
        }
    }
}
