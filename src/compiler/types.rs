//! Système de types de Kastel.
//!
//! Cette couche est indépendante de la VM : elle transforme les annotations
//! syntaxiques (`TypeExpr`) en types sémantiques (`Type`) et fournit les
//! opérations communes au TypeChecker : affichage, compatibilité,
//! promotion numérique et fusion de types pour l'inférence.

use std::fmt;

use crate::frontend::ast::TypeExpr;

use super::capability::Capability;

/// Type sémantique utilisé par le vérificateur statique.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Str,
    Bool,
    None,
    Array(Box<Type>),
    Dict(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    /// Type objet : `{ name: str, age: int }` (champs nommés, forme fixe).
    /// Typage STRUCTUREL : tout record qui possède au moins ces champs, avec
    /// des types compatibles, convient.
    Record(Vec<(String, Type)>),
    /// Union : `int | float`.
    Union(Vec<Type>),
    /// Fonction SURCHARGÉE par arité (`func f(a)` + `func f(a, b)`), telle
    /// qu'exportée par un module : un appel choisit la signature par arité
    /// et par type.
    Overloads(Vec<FunctionType>),
    /// `Set<T>` : ensemble d'éléments uniques de type `T`.
    Set(Box<Type>),
    /// `list`, `dict`, `tuple`, `set` non paramétrés.
    ArrayDynamic,
    DictDynamic,
    TupleDynamic,
    SetDynamic,
    Range,
    Function(FunctionType),
    Named(String),
    /// Paramètre de type d'une déclaration générique (`T`, `U`, ...).
    TypeParam(String),
    Generic {
        name: String,
        arguments: Vec<Type>,
    },
    /// Référence statique vers un module chargé par le resolver.
    Module(String),
    /// Absence d'information statique. Ce n'est jamais une erreur en soi.
    Dynamic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenericConstraint {
    Capability(Capability),
    Interface(Box<Type>),
}

impl GenericConstraint {
    pub fn name(&self) -> String {
        match self {
            Self::Capability(capability) => capability.to_string(),
            Self::Interface(interface) => interface.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionType {
    pub generic_params: Vec<String>,
    /// Contraintes sémantiques des paramètres génériques.
    /// Elles peuvent représenter une capability intrinsèque ou une interface
    /// Kastel utilisateur.
    pub generic_constraints: Vec<(String, Vec<GenericConstraint>)>,
    pub params: Vec<Type>,
    pub return_type: Box<Type>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericType {
    Int,
    Float,
}

impl NumericType {
    pub fn to_type(self) -> Type {
        match self {
            NumericType::Int => Type::Int,
            NumericType::Float => Type::Float,
        }
    }
}

impl Type {
    pub fn from_annotation(annotation: Option<&TypeExpr>) -> Type {
        match annotation {
            None => Type::Dynamic,
            Some(expr) => Self::from_type_expr(expr),
        }
    }

    /// Conversion sémantique d'un type syntaxique.
    ///
    /// Les noms de types inconnus sont volontairement conservés comme
    /// `Named(...)`. Cela permet les classes définies dans d'autres modules
    /// sans forcer le TypeChecker à connaître tout le graphe des imports.
    pub fn from_type_expr(expr: &TypeExpr) -> Type {
        match expr {
            TypeExpr::Named(name) => Self::from_name(name),
            TypeExpr::Generic { name, arguments } => Self::from_generic(name, arguments),

            TypeExpr::Union(members) => {
                Self::union_of(members.iter().map(Self::from_type_expr).collect())
            }

            TypeExpr::Record(fields) => Type::Record(
                fields
                    .iter()
                    .map(|(name, field)| (name.clone(), Self::from_type_expr(field)))
                    .collect(),
            ),
        }
    }

    /// Union normalisée : aplatie, sans doublon ; un seul membre = ce type ;
    /// un membre `Dynamic` rend toute l'union dynamique.
    pub fn union_of(types: Vec<Type>) -> Type {
        let mut members: Vec<Type> = Vec::new();

        for ty in types {
            match ty {
                Type::Dynamic => return Type::Dynamic,

                Type::Union(inner) => {
                    for member in inner {
                        if !members.contains(&member) {
                            members.push(member);
                        }
                    }
                }

                other => {
                    if !members.contains(&other) {
                        members.push(other);
                    }
                }
            }
        }

        match members.len() {
            0 => Type::Dynamic,
            1 => members.remove(0),
            _ => Type::Union(members),
        }
    }

    /// Membres d'une union ; un type non union est son propre unique membre.
    pub fn members(&self) -> Vec<Type> {
        match self {
            Type::Union(members) => members.clone(),
            other => vec![other.clone()],
        }
    }

    fn from_name(name: &str) -> Type {
        match name.to_ascii_lowercase().as_str() {
            "int" => Type::Int,
            "float" => Type::Float,
            "str" | "string" => Type::Str,
            "bool" | "boolean" => Type::Bool,
            "none" | "nil" | "null" => Type::None,
            "list" => Type::ArrayDynamic,
            "dict" => Type::DictDynamic,
            "tuple" => Type::TupleDynamic,
            "set" => Type::SetDynamic,
            "dynamic" | "any" => Type::Dynamic,
            _ => Type::Named(name.to_string()),
        }
    }

    fn from_generic(name: &str, arguments: &[TypeExpr]) -> Type {
        Self::build_generic(name, arguments.iter().map(Self::from_type_expr).collect())
    }

    /// Construit un type paramétré à partir d'arguments DÉJÀ convertis (le
    /// vérificateur résout d'abord les alias : `List<Person>`).
    pub(crate) fn build_generic(name: &str, arguments: Vec<Type>) -> Type {
        let normalized = name.to_ascii_lowercase();

        match normalized.as_str() {
            "list" if arguments.len() == 1 => Type::Array(Box::new(
                arguments.into_iter().next().expect("un argument de type"),
            )),

            "dict" | "map" if arguments.len() == 2 => {
                let mut arguments = arguments.into_iter();
                let key = arguments.next().expect("type de clé");
                let value = arguments.next().expect("type de valeur");

                Type::Dict(Box::new(key), Box::new(value))
            }

            "tuple" => Type::Tuple(arguments),

            "set" if arguments.len() == 1 => Type::Set(Box::new(
                arguments.into_iter().next().expect("un argument de type"),
            )),

            _ => Type::Generic {
                name: name.to_string(),
                arguments,
            },
        }
    }

    pub fn is_dynamic(&self) -> bool {
        matches!(self, Type::Dynamic)
    }

    pub fn numeric_kind(&self) -> Option<NumericType> {
        match self {
            Type::Int => Some(NumericType::Int),
            Type::Float => Some(NumericType::Float),
            _ => None,
        }
    }

    /// Retourne `true` si `actual` peut être affecté à `expected` sans
    /// déclencher d'erreur statique.
    pub fn is_assignable_to(
        &self,
        expected: &Type,
        parents: &impl Fn(&str) -> Vec<String>,
    ) -> bool {
        if self.is_dynamic() || expected.is_dynamic() {
            return true;
        }

        if self == expected {
            return true;
        }

        // Promotion numérique sûre : int -> float.
        if matches!((self, expected), (Type::Int, Type::Float)) {
            return true;
        }

        // Union attendue : la valeur doit convenir à AU MOINS un membre ;
        // une union fournie doit voir CHACUN de ses membres convenir.
        if let Type::Union(members) = expected {
            return match self {
                Type::Union(sources) => sources
                    .iter()
                    .all(|source| source.is_assignable_to(expected, parents)),

                _ => members
                    .iter()
                    .any(|member| self.is_assignable_to(member, parents)),
            };
        }

        if let Type::Union(sources) = self {
            return sources
                .iter()
                .all(|source| source.is_assignable_to(expected, parents));
        }

        match (self, expected) {
            // Une fonction surchargée convient à un type fonction si UNE de
            // ses signatures y convient.
            (Type::Overloads(signatures), Type::Function(_)) => {
                signatures.iter().any(|signature| {
                    Type::Function(signature.clone()).is_assignable_to(expected, parents)
                })
            }

            // Typage structurel : le record fourni doit avoir TOUS les champs
            // attendus, avec des types compatibles (champs en plus permis).
            (Type::Record(actual), Type::Record(expected)) => {
                expected.iter().all(|(name, expected_type)| {
                    actual
                        .iter()
                        .find(|(actual_name, _)| actual_name == name)
                        .is_some_and(|(_, actual_type)| {
                            actual_type.is_assignable_to(expected_type, parents)
                        })
                })
            }

            (Type::TypeParam(actual), Type::TypeParam(expected)) => actual == expected,

            (Type::TypeParam(_), _) | (_, Type::TypeParam(_)) => false,

            (Type::Named(actual), Type::Named(expected)) => {
                Self::is_named_subtype(actual, expected, parents)
            }

            (Type::Array(actual), Type::Array(expected)) => {
                actual.is_assignable_to(expected, parents)
            }

            (Type::Dict(actual_k, actual_v), Type::Dict(expected_k, expected_v)) => {
                actual_k.is_assignable_to(expected_k, parents)
                    && actual_v.is_assignable_to(expected_v, parents)
            }

            // Un tuple est compatible élément par élément (covariance) et
            // seulement à longueur égale : `(1, "a")` est un `Tuple<float, str>`
            // mais pas un `Tuple<int>`.
            (Type::Tuple(actual), Type::Tuple(expected)) => {
                actual.len() == expected.len()
                    && actual
                        .iter()
                        .zip(expected)
                        .all(|(actual, expected)| actual.is_assignable_to(expected, parents))
            }

            (Type::Set(actual), Type::Set(expected)) => actual.is_assignable_to(expected, parents),
            (Type::SetDynamic, Type::Set(_)) => true,
            (Type::Set(_), Type::SetDynamic) => true,

            (Type::ArrayDynamic, Type::Array(_)) => true,
            (Type::Array(_), Type::ArrayDynamic) => true,
            (Type::DictDynamic, Type::Dict(_, _)) => true,
            (Type::Dict(_, _), Type::DictDynamic) => true,
            (Type::TupleDynamic, Type::Tuple(_)) => true,
            (Type::Tuple(_), Type::TupleDynamic) => true,

            (Type::Function(actual), Type::Function(expected)) => {
                actual.generic_params == expected.generic_params
                    && actual.generic_constraints == expected.generic_constraints
                    && actual.params.len() == expected.params.len()
                    && actual
                        .params
                        .iter()
                        .zip(&expected.params)
                        .all(|(actual, expected)| {
                            actual == expected || actual.is_dynamic() || expected.is_dynamic()
                        })
                    && actual
                        .return_type
                        .is_assignable_to(&expected.return_type, parents)
            }

            (
                Type::Generic {
                    name: actual_name,
                    arguments: actual_args,
                },
                Type::Generic {
                    name: expected_name,
                    arguments: expected_args,
                },
            ) => {
                actual_name == expected_name
                    && actual_args.len() == expected_args.len()
                    && actual_args
                        .iter()
                        .zip(expected_args)
                        .all(|(actual, expected)| actual.is_assignable_to(expected, parents))
            }

            _ => false,
        }
    }

    fn is_named_subtype(
        actual: &str,
        expected: &str,
        parents: &impl Fn(&str) -> Vec<String>,
    ) -> bool {
        if actual == expected {
            return true;
        }

        let mut pending = parents(actual);
        let mut visited = std::collections::HashSet::new();

        while let Some(current) = pending.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }

            if current == expected {
                return true;
            }

            pending.extend(parents(&current));
        }

        false
    }

    /// Fusion pour l'inférence d'une expression qui peut avoir deux types.
    /// Une différence non résolue produit `Dynamic` au lieu de rendre un
    /// programme dynamique invalide.
    pub fn merge(&self, other: &Type) -> Type {
        if self == other {
            return self.clone();
        }

        if self.is_dynamic() || other.is_dynamic() {
            return Type::Dynamic;
        }

        // Deux tuples de même longueur se fusionnent élément par élément.
        if let (Type::Tuple(left), Type::Tuple(right)) = (self, other)
            && left.len() == right.len()
        {
            return Type::Tuple(
                left.iter()
                    .zip(right)
                    .map(|(left, right)| left.merge(right))
                    .collect(),
            );
        }

        match (self.numeric_kind(), other.numeric_kind()) {
            (Some(NumericType::Int), Some(NumericType::Int)) => Type::Int,
            (Some(_), Some(_)) => Type::Float,
            _ => Type::Dynamic,
        }
    }

    pub fn element_type(&self) -> Type {
        match self {
            Type::Array(element) => (**element).clone(),
            Type::ArrayDynamic => Type::Dynamic,
            Type::Tuple(elements) => elements
                .iter()
                .cloned()
                .reduce(|a, b| a.merge(&b))
                .unwrap_or(Type::Dynamic),
            Type::TupleDynamic => Type::Dynamic,
            Type::Set(element) => (**element).clone(),
            Type::SetDynamic => Type::Dynamic,
            Type::Range => Type::Float,
            Type::Dict(_, value) => (**value).clone(),
            Type::DictDynamic => Type::Dynamic,
            _ => Type::Dynamic,
        }
    }

    /// Nom de remplacement si `name` est une méthode SUPPRIMÉE de l'API
    /// standard des collections, pour un récepteur de ce type.
    pub fn renamed_member(&self, name: &str) -> Option<&'static str> {
        match (self, name) {
            (_, "to_iterator") if self.is_standard_collection() => Some("iter()"),

            // `length` pourrait être une clé de dict : on ne le signale
            // donc PAS sur les dicts (l'appel `d.length()` échoue à
            // l'exécution avec le même indice).
            (
                Type::Array(_)
                | Type::ArrayDynamic
                | Type::Tuple(_)
                | Type::TupleDynamic
                | Type::Set(_)
                | Type::SetDynamic
                | Type::Str,
                "length",
            ) => Some("size()"),

            (Type::Array(_) | Type::ArrayDynamic, "push") => Some("add(value)"),

            // `Array` s'appelle désormais `List`.
            (Type::Tuple(_) | Type::TupleDynamic | Type::Set(_) | Type::SetDynamic, "to_array") => {
                Some("to_list()")
            }

            (Type::Dict(_, _) | Type::DictDynamic, "has") => Some("contains(key)"),
            (Type::Dict(_, _) | Type::DictDynamic, "items") => Some("entries()"),

            _ => None,
        }
    }

    fn is_standard_collection(&self) -> bool {
        matches!(
            self,
            Type::Array(_)
                | Type::ArrayDynamic
                | Type::Dict(_, _)
                | Type::DictDynamic
                | Type::Tuple(_)
                | Type::TupleDynamic
                | Type::Set(_)
                | Type::SetDynamic
                | Type::Str
                | Type::Range
        )
    }

    /// Signature des méthodes STANDARD d'Array, Dict, Tuple, String et Range
    /// (`size`, `is_empty`, `contains`, `copy`, `clear`, `add`, `remove`,
    /// `get`, `set`, `keys`...), ou `None` pour toute autre méthode (qui
    /// reste alors dynamique).
    ///
    /// À n'utiliser que pour un APPEL : `d.size` sans parenthèses peut être
    /// une clé de dict. Doit rester alignée sur `stdlib::{array,dict,tuple,
    /// string}::dispatch_method` et sur `VirtualMachine::range_method`.
    pub fn collection_member_type(&self, name: &str) -> Option<Type> {
        if !self.is_standard_collection() {
            return None;
        }

        let method = |params: Vec<Type>, result: Type| {
            Some(Type::Function(FunctionType {
                generic_params: Vec::new(),
                generic_constraints: Vec::new(),
                params,
                return_type: Box::new(result),
            }))
        };

        // Commun à toutes les collections.
        match name {
            "size" => return method(vec![], Type::Int),
            "is_empty" => return method(vec![], Type::Bool),
            "to_string" => return method(vec![], Type::Str),
            "iter" => return method(vec![], Type::Dynamic),
            _ => {}
        }

        match self {
            Type::Array(_) | Type::ArrayDynamic => {
                let element = self.element_type();

                match name {
                    "contains" => method(vec![Type::Dynamic], Type::Bool),
                    "copy" => method(vec![], self.clone()),
                    "clear" => method(vec![], Type::None),
                    "add" | "remove" => method(vec![Type::Dynamic], Type::Bool),
                    "remove_at" | "get" => method(vec![Type::Dynamic], element),
                    "first" | "last" | "pop" => method(vec![], element),
                    "index_of" => method(vec![Type::Dynamic], Type::Int),
                    _ => None,
                }
            }

            Type::Dict(_, _) | Type::DictDynamic => {
                let key = self.key_type();
                let value = self.element_type();

                match name {
                    // Pour un dict, `contains` teste l'existence d'une CLÉ.
                    "contains" => method(vec![Type::Dynamic], Type::Bool),
                    "copy" => method(vec![], self.clone()),
                    "clear" => method(vec![], Type::None),
                    "get" | "remove" => method(vec![Type::Dynamic], value),
                    "set" => method(vec![Type::Dynamic, Type::Dynamic], Type::None),
                    "keys" => method(vec![], Type::Array(Box::new(key))),
                    "values" => method(vec![], Type::Array(Box::new(value))),
                    "entries" => method(
                        vec![],
                        Type::Array(Box::new(Type::Array(Box::new(Type::Dynamic)))),
                    ),
                    _ => None,
                }
            }

            // Un tuple est immuable : ni add, ni remove, ni clear, ni copy.
            Type::Tuple(_) | Type::TupleDynamic => {
                let element = self.element_type();

                match name {
                    "contains" => method(vec![Type::Dynamic], Type::Bool),
                    "get" => method(vec![Type::Dynamic], element),
                    "first" | "last" => method(vec![], element),
                    "index_of" => method(vec![Type::Dynamic], Type::Int),
                    "to_list" => method(vec![], Type::Array(Box::new(element))),
                    _ => None,
                }
            }

            Type::Str => match name {
                "contains" => method(vec![Type::Dynamic], Type::Bool),
                _ => None,
            },

            Type::Range => match name {
                "start" | "stop" | "step" => method(vec![], Type::Int),
                _ => None,
            },

            _ => None,
        }
    }

    /// Type des méthodes d'introspection d'un record (`p.keys()`, `p.copy()`),
    /// ou `None`. Un CHAMP de même nom l'emporte (voir `member_type`).
    pub fn record_method_type(&self, name: &str) -> Option<Type> {
        let Type::Record(fields) = self else {
            return None;
        };

        let method = |params: Vec<Type>, result: Type| {
            Some(Type::Function(FunctionType {
                generic_params: Vec::new(),
                generic_constraints: Vec::new(),
                params,
                return_type: Box::new(result),
            }))
        };

        match name {
            "keys" => method(vec![], Type::Array(Box::new(Type::Str))),

            "values" => method(
                vec![],
                Type::Array(Box::new(Type::union_of(
                    fields.iter().map(|(_, field)| field.clone()).collect(),
                ))),
            ),

            "entries" => method(
                vec![],
                Type::Array(Box::new(Type::Array(Box::new(Type::Dynamic)))),
            ),

            "copy" => method(vec![], self.clone()),
            "to_string" => method(vec![], Type::Str),

            _ => None,
        }
    }

    /// Type d'une méthode d'ensemble (`s.add`, `s.union`...), ou `None` si
    /// `self` n'est pas un ensemble ou si la méthode n'existe pas.
    ///
    /// Cette table doit rester alignée sur `stdlib::set::dispatch_method`.
    pub fn set_member_type(&self, name: &str) -> Option<Type> {
        let element = match self {
            Type::Set(element) => (**element).clone(),
            Type::SetDynamic => Type::Dynamic,
            _ => return None,
        };

        let method = |params: Vec<Type>, result: Type| {
            Some(Type::Function(FunctionType {
                generic_params: Vec::new(),
                generic_constraints: Vec::new(),
                params,
                return_type: Box::new(result),
            }))
        };

        match name {
            // `add` protège le type d'élément ; `contains`/`remove` acceptent
            // n'importe quelle valeur (tester l'appartenance est toujours
            // légitime).
            "add" => method(vec![element], Type::Bool),
            "remove" | "contains" => method(vec![Type::Dynamic], Type::Bool),

            "size" => method(vec![], Type::Int),
            "is_empty" => method(vec![], Type::Bool),
            "clear" => method(vec![], Type::None),
            "copy" => method(vec![], self.clone()),
            "to_list" => method(vec![], Type::Array(Box::new(element))),
            "to_string" => method(vec![], Type::Str),
            "iter" => method(vec![], Type::Dynamic),

            "union" | "intersection" | "difference" | "symmetric_difference" => {
                method(vec![Type::Dynamic], self.clone())
            }

            "is_subset" | "is_superset" | "equals" => method(vec![Type::Dynamic], Type::Bool),

            _ => None,
        }
    }

    pub fn key_type(&self) -> Type {
        match self {
            Type::Dict(key, _) => (**key).clone(),
            Type::DictDynamic => Type::Dynamic,
            _ => Type::Dynamic,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Str => write!(f, "str"),
            Type::Bool => write!(f, "bool"),
            Type::None => write!(f, "None"),
            Type::Array(element) => write!(f, "List<{element}>"),
            Type::Record(fields) => {
                write!(f, "{{ ")?;
                for (index, (name, field)) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{name}: {field}")?;
                }
                write!(f, " }}")
            }
            Type::Overloads(signatures) => {
                write!(f, "function ({} surcharges)", signatures.len())
            }
            Type::Union(members) => {
                for (index, member) in members.iter().enumerate() {
                    if index > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{member}")?;
                }
                Ok(())
            }
            Type::Dict(key, value) => write!(f, "Dict<{key}, {value}>"),
            Type::Tuple(elements) => {
                write!(f, "Tuple<")?;
                for (index, element) in elements.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{element}")?;
                }
                write!(f, ">")
            }
            Type::Set(element) => write!(f, "Set<{element}>"),
            Type::SetDynamic => write!(f, "Set"),
            Type::ArrayDynamic => write!(f, "List"),
            Type::DictDynamic => write!(f, "Dict"),
            Type::TupleDynamic => write!(f, "Tuple"),
            Type::Range => write!(f, "Range"),
            Type::Function(function) => {
                write!(f, "(")?;
                for (index, parameter) in function.params.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{parameter}")?;
                }
                write!(f, ") -> {}", function.return_type)
            }
            Type::Named(name) => write!(f, "{name}"),
            Type::TypeParam(name) => write!(f, "{name}"),
            Type::Generic { name, arguments } => {
                write!(f, "{name}<")?;
                for (index, argument) in arguments.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{argument}")?;
                }
                write!(f, ">")
            }
            Type::Module(_) => write!(f, "module"),
            Type::Dynamic => write!(f, "dynamic"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::TypeExpr;

    fn named(name: &str) -> TypeExpr {
        TypeExpr::Named(name.to_string())
    }

    #[test]
    fn maps_primitive_names() {
        assert_eq!(Type::from_annotation(Some(&named("int"))), Type::Int);
        assert_eq!(Type::from_annotation(Some(&named("float"))), Type::Float);
        assert_eq!(Type::from_annotation(Some(&named("str"))), Type::Str);
        assert_eq!(Type::from_annotation(Some(&named("bool"))), Type::Bool);
    }

    #[test]
    fn maps_missing_annotation_to_dynamic() {
        assert_eq!(Type::from_annotation(None), Type::Dynamic);
    }

    #[test]
    fn maps_generic_dict() {
        let expr = TypeExpr::Generic {
            name: "Dict".to_string(),
            arguments: vec![named("str"), named("int")],
        };

        assert_eq!(
            Type::from_type_expr(&expr),
            Type::Dict(Box::new(Type::Str), Box::new(Type::Int))
        );
    }

    #[test]
    fn unions_are_normalised_and_assignability_follows_membership() {
        let parents = |_name: &str| Vec::<String>::new();
        let number = Type::union_of(vec![Type::Int, Type::Float]);

        // Aplatie, sans doublon ; un seul membre = ce membre.
        assert_eq!(
            Type::union_of(vec![number.clone(), Type::Float, Type::Str]),
            Type::Union(vec![Type::Int, Type::Float, Type::Str])
        );
        assert_eq!(Type::union_of(vec![Type::Int, Type::Int]), Type::Int);
        assert_eq!(
            Type::union_of(vec![Type::Int, Type::Dynamic]),
            Type::Dynamic
        );

        assert!(Type::Int.is_assignable_to(&number, &parents));
        assert!(Type::Float.is_assignable_to(&number, &parents));
        assert!(!Type::Str.is_assignable_to(&number, &parents));

        // Une union fournie doit voir chaque membre convenir.
        assert!(number.is_assignable_to(&Type::Float, &parents));
        assert!(!number.is_assignable_to(&Type::Int, &parents));
        assert!(number.is_assignable_to(&number, &parents));

        assert_eq!(number.to_string(), "int | float");
    }

    #[test]
    fn records_are_structural_and_width_subtyping_applies() {
        let parents = |_name: &str| Vec::<String>::new();

        let person = Type::Record(vec![
            ("name".to_string(), Type::Str),
            ("age".to_string(), Type::Int),
        ]);
        let wider = Type::Record(vec![
            ("age".to_string(), Type::Int),
            ("name".to_string(), Type::Str),
            ("city".to_string(), Type::Str),
        ]);
        let missing = Type::Record(vec![("name".to_string(), Type::Str)]);

        // Ordre des champs indifférent ; champs en plus permis.
        assert!(wider.is_assignable_to(&person, &parents));
        assert!(!missing.is_assignable_to(&person, &parents));

        assert_eq!(person.to_string(), "{ name: str, age: int }");
    }

    #[test]
    fn tuples_are_covariant_and_length_checked() {
        let parents = |_name: &str| Vec::<String>::new();
        let int_str = Type::Tuple(vec![Type::Int, Type::Str]);
        let float_str = Type::Tuple(vec![Type::Float, Type::Str]);
        let single = Type::Tuple(vec![Type::Int]);

        assert!(int_str.is_assignable_to(&float_str, &parents));
        assert!(!float_str.is_assignable_to(&int_str, &parents));
        assert!(!int_str.is_assignable_to(&single, &parents));
        assert!(int_str.is_assignable_to(&Type::TupleDynamic, &parents));
    }

    #[test]
    fn tuples_merge_element_by_element() {
        let left = Type::Tuple(vec![Type::Int, Type::Int]);
        let right = Type::Tuple(vec![Type::Float, Type::Int]);

        assert_eq!(
            left.merge(&right),
            Type::Tuple(vec![Type::Float, Type::Int])
        );
    }

    #[test]
    fn module_types_are_equal_by_path() {
        let parents = |_name: &str| Vec::<String>::new();
        let left = Type::Module("/tmp/math.ks".into());
        let same = Type::Module("/tmp/math.ks".into());
        let other = Type::Module("/tmp/other.ks".into());

        assert!(left.is_assignable_to(&same, &parents));
        assert!(!left.is_assignable_to(&other, &parents));
    }
}
