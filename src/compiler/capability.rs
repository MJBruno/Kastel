//! Capabilities intrinsèques du système de types de Kastel.
//!
//! Une capability décrit le contrat intrinsèque associé à un opérateur
//! et sert uniquement de registre entre opérateurs, interfaces intrinsèques
//! et règles primitives. Les noms ne circulent plus comme des chaînes dans le
//! TypeChecker : ils sont normalisés ici et associés directement aux
//! opérateurs du langage et aux types primitifs qui les supportent.

use crate::frontend::ast::BinaryOp;

use super::types::{FunctionType, Type};

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

    pub fn operator_method_name(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Div => "div",
            Self::Mod => "mod",
            Self::Eq => "equals",
            Self::Ord => "compare",
            Self::BitAnd => "bitand",
            Self::BitOr => "bitor",
            Self::BitXor => "bitxor",
            Self::ShiftLeft => "shl",
            Self::ShiftRight => "shr",
        }
    }

    /// Nombre d'arguments génériques de l'interface intrinsèque.
    ///
    /// Les opérateurs arithmétiques et bitwise utilisent `Rhs, Output`.
    /// `Eq` et `Ord` ont un seul paramètre `Rhs` car leur résultat est fixé
    /// respectivement à `bool` et `int`.
    pub fn generic_arity(self) -> usize {
        match self {
            Self::Eq | Self::Ord => 1,
            _ => 2,
        }
    }

    /// Transforme une référence intrinsèque abrégée (`Add`) en contrat
    /// générique concret (`Add<Self, Self>` ou `Eq<Self>`).
    pub fn default_interface_type(self, self_type: &Type) -> Type {
        let arguments = match self {
            Self::Eq | Self::Ord => vec![self_type.clone()],
            _ => vec![self_type.clone(), self_type.clone()],
        };

        Type::Generic {
            name: self.name().to_string(),
            arguments,
        }
    }

    /// Construit la signature exigée par une interface intrinsèque
    /// instanciée.
    pub fn signature_for_interface(self, interface_type: &Type) -> Option<FunctionType> {
        let arguments = match interface_type {
            Type::Generic { name, arguments } if name == self.name() => arguments,
            Type::Named(name) if name == self.name() => return Some(self.bare_signature()),
            _ => return None,
        };

        if arguments.len() != self.generic_arity() {
            return None;
        }

        let (parameter, output) = match self {
            Self::Eq => (arguments[0].clone(), Type::Bool),
            Self::Ord => (arguments[0].clone(), Type::Int),
            _ => (arguments[0].clone(), arguments[1].clone()),
        };

        Some(FunctionType {
            generic_params: Vec::new(),
            generic_constraints: Vec::new(),
            params: vec![parameter],
            return_type: Box::new(output),
        })
    }

    fn bare_signature(self) -> FunctionType {
        let interface = self.default_interface_type(&Type::SelfType);
        self.signature_for_interface(&interface)
            .expect("une capability possède une signature intrinsèque")
    }

    /// Résultat intrinsèque d'un opérateur appliqué aux deux types concrets.
    /// `None` signifie que le couple d'opérandes n'est pas une opération
    /// intrinsèque valide ; le TypeChecker cherchera alors un contrat
    /// d'interface explicite.
    pub fn intrinsic_result(self, left: &Type, right: &Type) -> Option<Type> {
        use super::types::NumericType;

        match self {
            Self::Add => match (left, right) {
                (Type::Str, Type::Str) => Some(Type::Str),
                (Type::Int, Type::Int) => Some(Type::Int),
                (Type::Int, Type::Float)
                | (Type::Float, Type::Int)
                | (Type::Float, Type::Float) => Some(Type::Float),
                _ => None,
            },

            Self::Sub | Self::Mul => match (left.numeric_kind(), right.numeric_kind()) {
                (Some(NumericType::Int), Some(NumericType::Int)) => Some(Type::Int),
                (Some(_), Some(_)) => Some(Type::Float),
                _ => None,
            },

            Self::Div => match (left.numeric_kind(), right.numeric_kind()) {
                (Some(_), Some(_)) => Some(Type::Float),
                _ => None,
            },

            Self::Mod => match (left.numeric_kind(), right.numeric_kind()) {
                (Some(NumericType::Int), Some(NumericType::Int)) => Some(Type::Int),
                (Some(_), Some(_)) => Some(Type::Float),
                _ => None,
            },

            Self::Eq => {
                let left_primitive = matches!(
                    left,
                    Type::Int
                        | Type::Float
                        | Type::Bool
                        | Type::Str
                        | Type::None
                );
                let right_primitive = matches!(
                    right,
                    Type::Int
                        | Type::Float
                        | Type::Bool
                        | Type::Str
                        | Type::None
                );

                if left_primitive && right_primitive {
                    Some(Type::Bool)
                } else {
                    None
                }
            }

            Self::Ord => {
                if left.numeric_kind().is_some() && right.numeric_kind().is_some() {
                    Some(Type::Bool)
                } else {
                    None
                }
            }

            Self::BitAnd
            | Self::BitOr
            | Self::BitXor
            | Self::ShiftLeft
            | Self::ShiftRight => {
                if matches!(left, Type::Int) && matches!(right, Type::Int) {
                    Some(Type::Int)
                } else {
                    None
                }
            }
        }
    }

    pub fn from_operator(operator: &BinaryOp) -> Option<Self> {
        match operator {
            BinaryOp::Add => Some(Self::Add),
            BinaryOp::Subtract => Some(Self::Sub),
            BinaryOp::Multiply => Some(Self::Mul),
            BinaryOp::Divide => Some(Self::Div),
            BinaryOp::Modulo => Some(Self::Mod),
            BinaryOp::Equal | BinaryOp::NotEqual => Some(Self::Eq),
            BinaryOp::Is => None,
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
        assert_eq!(Capability::from_operator(&BinaryOp::Is), None);
        assert_eq!(Capability::Eq.operator_method_name(), "equals");
        assert_eq!(Capability::Ord.operator_method_name(), "compare");
    }

    #[test]
    fn heterogeneous_interfaces_have_explicit_rhs_and_output() {
        let div = Capability::Div.default_interface_type(&Type::Int);
        assert_eq!(
            div,
            Type::Generic {
                name: "Div".to_string(),
                arguments: vec![Type::Int, Type::Int],
            }
        );

        let signature = Capability::Div.signature_for_interface(&Type::Generic {
            name: "Div".to_string(),
            arguments: vec![Type::Int, Type::Float],
        }).expect("signature Div<int,float>");

        assert_eq!(signature.params, vec![Type::Int]);
        assert_eq!(*signature.return_type, Type::Float);
    }

    #[test]
    fn intrinsic_results_preserve_numeric_output_rules() {
        assert_eq!(
            Capability::Div.intrinsic_result(&Type::Int, &Type::Int),
            Some(Type::Float)
        );
        assert_eq!(
            Capability::Add.intrinsic_result(&Type::Int, &Type::Float),
            Some(Type::Float)
        );
        assert_eq!(
            Capability::Add.intrinsic_result(&Type::Str, &Type::Str),
            Some(Type::Str)
        );
        assert_eq!(
            Capability::BitAnd.intrinsic_result(&Type::Int, &Type::Int),
            Some(Type::Int)
        );
    }


}