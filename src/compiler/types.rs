//! Système de types de Kastel.
//!
//! Cette couche est indépendante de la VM : elle transforme les annotations
//! syntaxiques (`TypeExpr`) en types sémantiques (`Type`) et fournit les
//! opérations communes au TypeChecker : affichage, compatibilité,
//! promotion numérique et fusion de types pour l'inférence.

use std::fmt;

use crate::frontend::ast::TypeExpr;

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
    /// `array`, `dict`, `tuple` non paramétrés.
    ArrayDynamic,
    DictDynamic,
    TupleDynamic,
    Range,
    Function(FunctionType),
    Named(String),
    Generic { name: String, arguments: Vec<Type> },
    /// Référence statique vers un module chargé par le resolver.
    Module(String),
    /// Absence d'information statique. Ce n'est jamais une erreur en soi.
    Dynamic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionType {
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
        }
    }

    fn from_name(name: &str) -> Type {
        match name.to_ascii_lowercase().as_str() {
            "int" => Type::Int,
            "float" => Type::Float,
            "str" | "string" => Type::Str,
            "bool" | "boolean" => Type::Bool,
            "none" | "nil" | "null" => Type::None,
            "array" => Type::ArrayDynamic,
            "dict" => Type::DictDynamic,
            "tuple" => Type::TupleDynamic,
            "dynamic" | "any" => Type::Dynamic,
            _ => Type::Named(name.to_string()),
        }
    }

    fn from_generic(name: &str, arguments: &[TypeExpr]) -> Type {
        let normalized = name.to_ascii_lowercase();

        match normalized.as_str() {
            "array" | "list" => {
                if arguments.len() == 1 {
                    Type::Array(Box::new(Self::from_type_expr(&arguments[0])))
                } else {
                    Type::Generic {
                        name: name.to_string(),
                        arguments: arguments.iter().map(Self::from_type_expr).collect(),
                    }
                }
            }

            "dict" | "map" => {
                if arguments.len() == 2 {
                    Type::Dict(
                        Box::new(Self::from_type_expr(&arguments[0])),
                        Box::new(Self::from_type_expr(&arguments[1])),
                    )
                } else {
                    Type::Generic {
                        name: name.to_string(),
                        arguments: arguments.iter().map(Self::from_type_expr).collect(),
                    }
                }
            }

            "tuple" => Type::Tuple(arguments.iter().map(Self::from_type_expr).collect()),

            _ => Type::Generic {
                name: name.to_string(),
                arguments: arguments.iter().map(Self::from_type_expr).collect(),
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
    pub fn is_assignable_to(&self, expected: &Type, parents: &impl Fn(&str) -> Vec<String>) -> bool {
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

        match (self, expected) {
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

            (Type::ArrayDynamic, Type::Array(_)) => true,
            (Type::Array(_), Type::ArrayDynamic) => true,
            (Type::DictDynamic, Type::Dict(_, _)) => true,
            (Type::Dict(_, _), Type::DictDynamic) => true,
            (Type::TupleDynamic, Type::Tuple(_)) => true,
            (Type::Tuple(_), Type::TupleDynamic) => true,

            (Type::Function(actual), Type::Function(expected)) => {
                actual.params.len() == expected.params.len()
                    && actual
                        .params
                        .iter()
                        .zip(&expected.params)
                        .all(|(actual, expected)| actual == expected || actual.is_dynamic() || expected.is_dynamic())
                    && actual.return_type.is_assignable_to(&expected.return_type, parents)
            }

            (Type::Generic { name: actual_name, arguments: actual_args }, Type::Generic { name: expected_name, arguments: expected_args }) => {
                actual_name == expected_name
                    && actual_args.len() == expected_args.len()
                    && actual_args.iter().zip(expected_args).all(|(actual, expected)| {
                        actual.is_assignable_to(expected, parents)
                    })
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
            Type::Tuple(elements) => {
                elements
                    .iter()
                    .cloned()
                    .reduce(|a, b| a.merge(&b))
                    .unwrap_or(Type::Dynamic)
            }
            Type::TupleDynamic => Type::Dynamic,
            Type::Range => Type::Float,
            Type::Dict(_, value) => (**value).clone(),
            Type::DictDynamic => Type::Dynamic,
            _ => Type::Dynamic,
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
            Type::Array(element) => write!(f, "Array<{element}>"),
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
            Type::ArrayDynamic => write!(f, "Array"),
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
    fn module_types_are_equal_by_path() {
        let parents = |_name: &str| Vec::<String>::new();
        let left = Type::Module("/tmp/math.ks".into());
        let same = Type::Module("/tmp/math.ks".into());
        let other = Type::Module("/tmp/other.ks".into());

        assert!(left.is_assignable_to(&same, &parents));
        assert!(!left.is_assignable_to(&other, &parents));
    }
}
