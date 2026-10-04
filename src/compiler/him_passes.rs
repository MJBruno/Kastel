//! Passes de transformation de la HIM.
//!
//! Les passes travaillent uniquement sur la représentation intermédiaire
//! possédée (`him.rs`). Elles ne dépendent ni du parser ni de l'AST.

use std::cmp::Ordering;

use super::him::{AssignmentTarget, BinaryOp, Expression, Literal, MatchArm, Statement};

/// Contrat commun d'une passe de transformation HIM.
pub(crate) trait HimPass {
    /// Transforme le module en place.
    fn run(&self, statements: &mut Vec<Statement>);
}

/// Exécute les passes HIM activées par défaut.
pub(crate) fn run_default_passes(statements: &mut Vec<Statement>) {
    let passes: [&dyn HimPass; 1] = [&ConstantFolder];

    for pass in passes {
        pass.run(statements);
    }
}

/// Replie les expressions entièrement constantes.
///
/// Cette passe est volontairement conservatrice :
/// - aucun appel, accès membre, allocation ou opération utilisateur n'est
///   transformé ;
/// - les divisions/modulos par zéro restent dans le bytecode ;
/// - les opérations entières ne sont pliées que si elles ne débordent pas ;
/// - les comparaisons entier/flottant suivent la règle exacte du VM.
pub(crate) struct ConstantFolder;

impl HimPass for ConstantFolder {
    fn run(&self, statements: &mut Vec<Statement>) {
        for statement in statements {
            fold_statement(statement);
        }
    }
}

fn fold_statement(statement: &mut Statement) {
    match statement {
        Statement::Positioned { statement, .. } => fold_statement(statement),

        Statement::Let { value, .. } => fold_expression(value),
        Statement::Assignment { target, value } => {
            fold_assignment_target(target);
            fold_expression(value);
        }
        Statement::Expression { expression } | Statement::Throw { value: expression } => {
            fold_expression(expression);
        }

        Statement::Block(statements) => {
            for statement in statements {
                fold_statement(statement);
            }
        }

        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            fold_expression(condition);
            for statement in then_branch {
                fold_statement(statement);
            }
            if let Some(branch) = else_branch {
                for statement in branch {
                    fold_statement(statement);
                }
            }
        }

        Statement::While { condition, body } => {
            fold_expression(condition);
            for statement in body {
                fold_statement(statement);
            }
        }

        Statement::ForIn { iterable, body, .. } => {
            fold_expression(iterable);
            for statement in body {
                fold_statement(statement);
            }
        }

        Statement::Match { value, arms } => {
            fold_expression(value);
            for arm in arms {
                fold_match_arm(arm);
            }
        }

        Statement::Try {
            try_body,
            catch_body,
            finally_body,
            ..
        } => {
            for statement in try_body {
                fold_statement(statement);
            }
            if let Some(body) = catch_body {
                for statement in body {
                    fold_statement(statement);
                }
            }
            if let Some(body) = finally_body {
                for statement in body {
                    fold_statement(statement);
                }
            }
        }

        Statement::Function { body, .. } => {
            for statement in body {
                fold_statement(statement);
            }
        }

        Statement::Return { value } => {
            if let Some(value) = value {
                fold_expression(value);
            }
        }

        Statement::Export { statement } => fold_statement(statement),

        Statement::Class { fields, methods, .. } => {
            for field in fields {
                if let Some(initializer) = &mut field.initializer {
                    fold_expression(initializer);
                }
            }
            for method in methods {
                for statement in &mut method.body {
                    fold_statement(statement);
                }
            }
        }

        Statement::Enum { methods, .. } => {
            for method in methods {
                for statement in &mut method.body {
                    fold_statement(statement);
                }
            }
        }

        Statement::Import { .. }
        | Statement::FromImport { .. }
        | Statement::TypeAlias { .. }
        | Statement::Interface { .. }
        | Statement::Break
        | Statement::Continue => {}
    }
}

fn fold_assignment_target(target: &mut AssignmentTarget) {
    match target {
        AssignmentTarget::Variable(_) => {}
        AssignmentTarget::Index { object, index } => {
            fold_expression(object);
            fold_expression(index);
        }
        AssignmentTarget::Member { object, .. } => fold_expression(object),
    }
}

fn fold_match_arm(arm: &mut MatchArm) {
    if let Some(guard) = &mut arm.guard {
        fold_expression(guard);
    }
    for statement in &mut arm.body {
        fold_statement(statement);
    }
}

fn fold_expression(expression: &mut Expression) {
    match expression {
        Expression::Literal(_)
        | Expression::Variable(_)
        | Expression::SelfValue => {}

        Expression::Unary { operator, right, .. } => {
            fold_expression(right);

            let replacement = match right.as_ref() {
                Expression::Literal(literal) => fold_unary(*operator, literal),
                _ => None,
            };

            if let Some(result) = replacement {
                *expression = Expression::Literal(result);
            }
        }

        Expression::Binary {
            left,
            operator,
            right,
            ..
        } => {
            fold_expression(left);
            fold_expression(right);

            let replacement = match (left.as_ref(), right.as_ref()) {
                (Expression::Literal(left), Expression::Literal(right)) => {
                    fold_binary(*operator, left, right)
                }
                _ => None,
            };

            if let Some(result) = replacement {
                *expression = Expression::Literal(result);
                return;
            }

            // `&&` / `||` renvoient l'un de leurs opérandes. Lorsque gauche
            // est constant et décide déjà le résultat, droite peut être
            // éliminée sans changer l'ordre d'évaluation.
            let replacement = match (operator, left.as_ref()) {
                (BinaryOp::And, Expression::Literal(left)) if !literal_truthy(left) => {
                    Some(left.clone())
                }
                (BinaryOp::Or, Expression::Literal(left)) if literal_truthy(left) => {
                    Some(left.clone())
                }
                _ => None,
            };

            if let Some(result) = replacement {
                *expression = Expression::Literal(result);
            }
        }

        Expression::Function { body, .. } => {
            for statement in body {
                fold_statement(statement);
            }
        }

        Expression::Call {
            callee, arguments, ..
        } => {
            fold_expression(callee);
            for argument in arguments {
                fold_expression(argument);
            }
        }

        Expression::Member { object, .. } => fold_expression(object),

        Expression::Index { object, index, .. } => {
            fold_expression(object);
            fold_expression(index);
        }

        Expression::New { arguments, .. } => {
            for argument in arguments {
                fold_expression(argument);
            }
        }

        Expression::Array(elements) | Expression::Tuple(elements) => {
            for element in elements {
                fold_expression(element);
            }
        }

        Expression::Dict(fields) | Expression::Record(fields) => {
            for (_, value) in fields {
                fold_expression(value);
            }
        }

        Expression::Try(expression) | Expression::Await(expression) => {
            fold_expression(expression);
        }

        Expression::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            fold_expression(condition);
            fold_expression(then_expr);
            fold_expression(else_expr);

            let replacement = match condition.as_ref() {
                Expression::Literal(condition) if literal_truthy(condition) => {
                    Some((**then_expr).clone())
                }
                Expression::Literal(_) => Some((**else_expr).clone()),
                _ => None,
            };

            if let Some(result) = replacement {
                *expression = result;
            }
        }
    }
}

fn fold_unary(operator: super::him::UnaryOp, literal: &Literal) -> Option<Literal> {
    match operator {
        super::him::UnaryOp::Negate => match literal {
            Literal::Integer(value) => value.checked_neg().map(Literal::Integer),
            Literal::Float(value) => Some(Literal::Float(-value)),
            _ => None,
        },
        super::him::UnaryOp::Not => Some(Literal::Bool(!literal_truthy(literal))),
        super::him::UnaryOp::BitNot => match literal {
            Literal::Integer(value) => Some(Literal::Integer(!value)),
            _ => None,
        },
    }
}

fn fold_binary(operator: BinaryOp, left: &Literal, right: &Literal) -> Option<Literal> {
    match operator {
        BinaryOp::Add => fold_add(left, right),
        BinaryOp::Subtract => fold_numeric(left, right, NumericFold::Subtract),
        BinaryOp::Multiply => fold_numeric(left, right, NumericFold::Multiply),
        BinaryOp::Divide => fold_divide(left, right),
        BinaryOp::Modulo => fold_modulo(left, right),

        BinaryOp::Equal => Some(Literal::Bool(literal_equals(left, right))),
        BinaryOp::NotEqual => Some(Literal::Bool(!literal_equals(left, right))),

        BinaryOp::Less => fold_compare(left, right, |ordering| ordering == Ordering::Less),
        BinaryOp::LessEqual => fold_compare(left, right, |ordering| ordering != Ordering::Greater),
        BinaryOp::Greater => {
            fold_compare(left, right, |ordering| ordering == Ordering::Greater)
        }
        BinaryOp::GreaterEqual => fold_compare(left, right, |ordering| ordering != Ordering::Less),

        BinaryOp::And => {
            if literal_truthy(left) {
                Some(right.clone())
            } else {
                Some(left.clone())
            }
        }
        BinaryOp::Or => {
            if literal_truthy(left) {
                Some(left.clone())
            } else {
                Some(right.clone())
            }
        }

        BinaryOp::BitAnd => fold_bitwise(left, right, |a, b| a & b),
        BinaryOp::BitOr => fold_bitwise(left, right, |a, b| a | b),
        BinaryOp::BitXor => fold_bitwise(left, right, |a, b| a ^ b),
        BinaryOp::ShiftLeft => fold_shift(left, right, true),
        BinaryOp::ShiftRight => fold_shift(left, right, false),

        // `is` dépend d'une valeur runtime de classe/interface/type.
        BinaryOp::Is => None,
    }
}

fn fold_add(left: &Literal, right: &Literal) -> Option<Literal> {
    match (left, right) {
        (Literal::String(a), Literal::String(b)) => Some(Literal::String(format!("{a}{b}"))),
        _ => fold_numeric(left, right, NumericFold::Add),
    }
}

#[derive(Clone, Copy)]
enum NumericFold {
    Add,
    Subtract,
    Multiply,
}

fn fold_numeric(left: &Literal, right: &Literal, operation: NumericFold) -> Option<Literal> {
    match (left, right) {
        (Literal::Integer(a), Literal::Integer(b)) => {
            let result = match operation {
                NumericFold::Add => a.checked_add(*b),
                NumericFold::Subtract => a.checked_sub(*b),
                NumericFold::Multiply => a.checked_mul(*b),
            }?;
            Some(Literal::Integer(result))
        }

        (Literal::Integer(a), Literal::Float(b)) => {
            Some(Literal::Float(apply_float(*a as f64, *b, operation)))
        }
        (Literal::Float(a), Literal::Integer(b)) => {
            Some(Literal::Float(apply_float(*a, *b as f64, operation)))
        }
        (Literal::Float(a), Literal::Float(b)) => {
            Some(Literal::Float(apply_float(*a, *b, operation)))
        }

        _ => None,
    }
}

fn apply_float(a: f64, b: f64, operation: NumericFold) -> f64 {
    match operation {
        NumericFold::Add => a + b,
        NumericFold::Subtract => a - b,
        NumericFold::Multiply => a * b,
    }
}

fn fold_divide(left: &Literal, right: &Literal) -> Option<Literal> {
    match (left, right) {
        (Literal::Integer(a), Literal::Integer(b)) if *b != 0 => {
            Some(Literal::Float(*a as f64 / *b as f64))
        }
        (Literal::Integer(a), Literal::Float(b)) if *b != 0.0 => {
            Some(Literal::Float(*a as f64 / *b))
        }
        (Literal::Float(a), Literal::Integer(b)) if *b != 0 => {
            Some(Literal::Float(*a / *b as f64))
        }
        (Literal::Float(a), Literal::Float(b)) if *b != 0.0 => {
            Some(Literal::Float(*a / *b))
        }
        _ => None,
    }
}

fn fold_modulo(left: &Literal, right: &Literal) -> Option<Literal> {
    match (left, right) {
        (Literal::Integer(a), Literal::Integer(b)) if *b != 0 => {
            if *a == i64::MIN && *b == -1 {
                Some(Literal::Integer(0))
            } else {
                Some(Literal::Integer(*a % *b))
            }
        }
        (Literal::Integer(a), Literal::Float(b)) if *b != 0.0 => {
            Some(Literal::Float((*a as f64) % *b))
        }
        (Literal::Float(a), Literal::Integer(b)) if *b != 0 => {
            Some(Literal::Float(*a % *b as f64))
        }
        (Literal::Float(a), Literal::Float(b)) if *b != 0.0 => {
            Some(Literal::Float(*a % *b))
        }
        _ => None,
    }
}

fn fold_bitwise(
    left: &Literal,
    right: &Literal,
    operation: impl FnOnce(i64, i64) -> i64,
) -> Option<Literal> {
    match (left, right) {
        (Literal::Integer(a), Literal::Integer(b)) => {
            Some(Literal::Integer(operation(*a, *b)))
        }
        _ => None,
    }
}

fn fold_shift(left: &Literal, right: &Literal, left_shift: bool) -> Option<Literal> {
    let (Literal::Integer(value), Literal::Integer(amount)) = (left, right) else {
        return None;
    };

    if !(0..64).contains(amount) {
        // Le VM produira InvalidShiftAmount à l'exécution.
        return None;
    }

    let amount = *amount as u32;
    let result = if left_shift {
        value.wrapping_shl(amount)
    } else {
        value.wrapping_shr(amount)
    };

    Some(Literal::Integer(result))
}

fn fold_compare(
    left: &Literal,
    right: &Literal,
    predicate: impl FnOnce(Ordering) -> bool,
) -> Option<Literal> {
    let ordering = numeric_ordering(left, right)?;
    Some(Literal::Bool(ordering.map(predicate).unwrap_or(false)))
}

fn numeric_ordering(left: &Literal, right: &Literal) -> Option<Option<Ordering>> {
    match (left, right) {
        (Literal::Integer(a), Literal::Integer(b)) => Some(Some(a.cmp(b))),
        (Literal::Float(a), Literal::Float(b)) => Some(a.partial_cmp(b)),
        (Literal::Integer(a), Literal::Float(b)) => Some(compare_integer_float(*a, *b)),
        (Literal::Float(a), Literal::Integer(b)) => {
            Some(compare_integer_float(*b, *a).map(Ordering::reverse))
        }
        _ => None,
    }
}

/// Même comparaison exacte entier/flottant que `Value::compare_integer_float`.
fn compare_integer_float(integer: i64, float: f64) -> Option<Ordering> {
    const I64_MAX_EXCLUSIVE: f64 = 9_223_372_036_854_775_808.0;

    if float.is_nan() {
        return None;
    }

    if float >= I64_MAX_EXCLUSIVE {
        return Some(Ordering::Less);
    }

    if float < -I64_MAX_EXCLUSIVE {
        return Some(Ordering::Greater);
    }

    let truncated = float.trunc();

    match integer.cmp(&(truncated as i64)) {
        Ordering::Equal => {
            let fraction = float - truncated;
            if fraction > 0.0 {
                Some(Ordering::Less)
            } else if fraction < 0.0 {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Equal)
            }
        }
        other => Some(other),
    }
}

fn literal_equals(left: &Literal, right: &Literal) -> bool {
    match (left, right) {
        (Literal::Integer(a), Literal::Integer(b)) => a == b,
        (Literal::Integer(a), Literal::Float(b)) => integer_equals_float(*a, *b),
        (Literal::Float(a), Literal::Integer(b)) => integer_equals_float(*b, *a),
        (Literal::Float(a), Literal::Float(b)) => a == b,
        (Literal::String(a), Literal::String(b)) => a == b,
        (Literal::Bool(a), Literal::Bool(b)) => a == b,
        (Literal::None, Literal::None) => true,
        _ => false,
    }
}

fn integer_equals_float(integer: i64, float: f64) -> bool {
    const I64_MAX_EXCLUSIVE: f64 = 9_223_372_036_854_775_808.0;

    float.fract() == 0.0
        && float >= i64::MIN as f64
        && float < I64_MAX_EXCLUSIVE
        && (float as i64) == integer
}

fn literal_truthy(literal: &Literal) -> bool {
    match literal {
        Literal::None => false,
        Literal::Bool(value) => *value,
        Literal::Integer(value) => *value != 0,
        Literal::Float(value) => *value != 0.0 && !value.is_nan(),
        Literal::String(value) => !value.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn literal(expression: Expression) -> Literal {
        match expression {
            Expression::Literal(value) => value,
            other => panic!("literal attendue, reçu: {other:?}"),
        }
    }

    #[test]
    fn folds_nested_integer_arithmetic() {
        let mut expression = Expression::Binary {
            left: Box::new(Expression::Literal(Literal::Integer(2))),
            operator: BinaryOp::Multiply,
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(Literal::Integer(3))),
                operator: BinaryOp::Add,
                right: Box::new(Expression::Literal(Literal::Integer(4))),
                line: 1,
                column: 5,
            }),
            line: 1,
            column: 1,
        };

        fold_expression(&mut expression);

        assert!(matches!(expression, Expression::Literal(Literal::Integer(14))));
    }

    #[test]
    fn folds_string_concatenation() {
        let result = fold_binary(
            BinaryOp::Add,
            &Literal::String("Ka".into()),
            &Literal::String("stel".into()),
        )
        .expect("concaténation attendue");

        assert_eq!(literal(Expression::Literal(result)), Literal::String("Kastel".into()));
    }

    #[test]
    fn never_folds_division_by_zero() {
        assert!(fold_binary(
            BinaryOp::Divide,
            &Literal::Integer(10),
            &Literal::Integer(0),
        )
        .is_none());
    }

    #[test]
    fn folds_logical_literals_with_operand_semantics() {
        let and_result = fold_binary(
            BinaryOp::And,
            &Literal::Integer(0),
            &Literal::String("non atteint".into()),
        )
        .expect("résultat constant attendu");
        assert_eq!(and_result, Literal::Integer(0));

        let or_result = fold_binary(
            BinaryOp::Or,
            &Literal::Integer(1),
            &Literal::String("non atteint".into()),
        )
        .expect("résultat constant attendu");
        assert_eq!(or_result, Literal::Integer(1));
    }

    #[test]
    fn folds_exact_integer_float_equality() {
        assert!(literal_equals(
            &Literal::Integer(9_007_199_254_740_992),
            &Literal::Float(9_007_199_254_740_992.0),
        ));
        assert!(!literal_equals(
            &Literal::Integer(9_007_199_254_740_993),
            &Literal::Float(9_007_199_254_740_992.0),
        ));
    }

    #[test]
    fn folds_nan_comparisons_to_false() {
        let result = fold_binary(
            BinaryOp::LessEqual,
            &Literal::Float(f64::NAN),
            &Literal::Float(1.0),
        )
        .expect("comparaison numérique attendue");

        assert_eq!(result, Literal::Bool(false));
    }

    #[test]
    fn folds_ternary_after_folding_condition() {
        let mut expression = Expression::Ternary {
            condition: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(Literal::Integer(2))),
                operator: BinaryOp::Less,
                right: Box::new(Expression::Literal(Literal::Integer(3))),
                line: 1,
                column: 1,
            }),
            then_expr: Box::new(Expression::Literal(Literal::String("yes".into()))),
            else_expr: Box::new(Expression::Literal(Literal::String("no".into()))),
        };

        fold_expression(&mut expression);

        assert_eq!(literal(expression), Literal::String("yes".into()));
    }
}
