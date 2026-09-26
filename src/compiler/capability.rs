//! Capabilities intrinsèques du système de types de Kastel.
//!
//! Une capability décrit une opération que le vérificateur peut exiger d'un
//! paramètre générique. Les noms ne circulent plus comme des chaînes dans le
//! TypeChecker : ils sont normalisés ici et associés directement aux
//! opérateurs du langage et aux types primitifs qui les supportent.

use crate::frontend::ast::BinaryOp;

use super::types::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ord,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

impl Capability {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Add" => Some(Self::Add),
            "Sub" => Some(Self::Sub),
            "Mul" => Some(Self::Mul),
            "Div" => Some(Self::Div),
            "Mod" => Some(Self::Mod),
            "Eq" => Some(Self::Eq),
            "Ord" => Some(Self::Ord),
            "BitAnd" => Some(Self::BitAnd),
            "BitOr" => Some(Self::BitOr),
            "BitXor" => Some(Self::BitXor),
            "ShiftLeft" => Some(Self::ShiftLeft),
            "ShiftRight" => Some(Self::ShiftRight),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Add => "Add",
            Self::Sub => "Sub",
            Self::Mul => "Mul",
            Self::Div => "Div",
            Self::Mod => "Mod",
            Self::Eq => "Eq",
            Self::Ord => "Ord",
            Self::BitAnd => "BitAnd",
            Self::BitOr => "BitOr",
            Self::BitXor => "BitXor",
            Self::ShiftLeft => "ShiftLeft",
            Self::ShiftRight => "ShiftRight",
        }
    }

    /// Nom de la méthode d'instance qu'une CLASSE (pas un `TypeParam`) peut
    /// définir pour surcharger l'opérateur correspondant (`class Money {
    /// func add(other: Money) -> Money { ... } }` rend `a + b` valide entre
    /// deux `Money`). `None` pour `Eq`/`Ord` : l'égalité structurelle est
    /// déjà universelle, et l'ordre (`<`, `<=`, `>`, `>=`) demande une
    /// convention d'appel différente (une seule méthode pour quatre
    /// opérateurs), hors du périmètre de cette surcharge un-opérateur/une-
    /// méthode.
    ///
    /// Implémenter une interface portant le même nom que la capability
    /// (ex. `interface Add { func add(other: Money) -> Money; }`) est une
    /// pure convention de lisibilité pour l'utilisateur Kastel : seule la
    /// présence de la méthode compte pour la résolution de l'opérateur, pas
    /// le nom de l'interface.
    pub fn operator_method_name(self) -> Option<&'static str> {
        match self {
            Self::Add => Some("add"),
            Self::Sub => Some("sub"),
            Self::Mul => Some("mul"),
            Self::Div => Some("div"),
            Self::Mod => Some("mod"),
            Self::BitAnd => Some("bitand"),
            Self::BitOr => Some("bitor"),
            Self::BitXor => Some("bitxor"),
            Self::ShiftLeft => Some("shl"),
            Self::ShiftRight => Some("shr"),
            Self::Eq | Self::Ord => None,
        }
    }

    pub fn from_operator(operator: &BinaryOp) -> Option<Self> {
        match operator {
            BinaryOp::Add => Some(Self::Add),
            BinaryOp::Subtract => Some(Self::Sub),
            BinaryOp::Multiply => Some(Self::Mul),
            BinaryOp::Divide => Some(Self::Div),
            BinaryOp::Modulo => Some(Self::Mod),
            BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::Is => Some(Self::Eq),
            BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual => Some(Self::Ord),
            BinaryOp::BitAnd => Some(Self::BitAnd),
            BinaryOp::BitOr => Some(Self::BitOr),
            BinaryOp::BitXor => Some(Self::BitXor),
            BinaryOp::ShiftLeft => Some(Self::ShiftLeft),
            BinaryOp::ShiftRight => Some(Self::ShiftRight),
            BinaryOp::And | BinaryOp::Or => None,
        }
    }

    /// Vérifie le support intrinsèque par un type concret.
    ///
    /// `Dynamic` reste permissif conformément au typage gradué de Kastel.
    pub fn is_satisfied_by(self, ty: &Type) -> bool {
        if ty.is_dynamic() {
            return true;
        }

        match self {
            Self::Add => matches!(ty, Type::Int | Type::Float | Type::Str),
            Self::Sub | Self::Mul | Self::Div | Self::Mod => {
                matches!(ty, Type::Int | Type::Float)
            }
            Self::Eq => true,
            Self::Ord => matches!(ty, Type::Int | Type::Float),
            Self::BitAnd | Self::BitOr | Self::BitXor | Self::ShiftLeft | Self::ShiftRight => {
                matches!(ty, Type::Int)
            }
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::BinaryOp;

    #[test]
    fn operator_mapping_is_centralized() {
        assert_eq!(Capability::from_operator(&BinaryOp::Add), Some(Capability::Add));
        assert_eq!(Capability::from_operator(&BinaryOp::BitAnd), Some(Capability::BitAnd));
        assert_eq!(Capability::from_operator(&BinaryOp::ShiftLeft), Some(Capability::ShiftLeft));
        assert_eq!(Capability::from_operator(&BinaryOp::And), None);
    }

    #[test]
    fn primitive_support_is_centralized() {
        assert!(Capability::Add.is_satisfied_by(&Type::Int));
        assert!(Capability::Add.is_satisfied_by(&Type::Str));
        assert!(!Capability::Sub.is_satisfied_by(&Type::Str));
        assert!(Capability::BitAnd.is_satisfied_by(&Type::Int));
        assert!(!Capability::BitAnd.is_satisfied_by(&Type::Bool));
    }
}