use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use super::him::*;
use crate::runtime::value::Value;

use super::{
    builtin_types::Intrinsic,
    call_metadata::{CallTarget, ResolvedCall, ResolvedMember},
    compiler::{Compiler, MAX_EXPRESSION_DEPTH},
};

impl Compiler {
    // ============================================================
    //                      EXPRESSION
    // ============================================================

    pub(crate) fn compile_expression(&mut self, expr: &Expression) -> Result<(), CompileError> {
        if self.expression_depth >= MAX_EXPRESSION_DEPTH {
            return Err(CompileError::ExpressionTooDeep {
                limit: MAX_EXPRESSION_DEPTH,
            });
        }

        self.expression_depth += 1;
        let result = self.compile_expression_inner(expr);
        self.expression_depth -= 1;

        result
    }

    fn compile_expression_inner(&mut self, expr: &Expression) -> Result<(), CompileError> {
        match expr {
            Expression::Literal(value) => {
                let value = match value {
                    Literal::Integer(v) => Value::Integer(*v),
                    Literal::Float(v) => Value::Float(*v),
                    Literal::String(v) => Value::new_string(v.clone()),
                    Literal::Bool(v) => Value::Boolean(*v),
                    Literal::None => Value::None,
                };

                let constant = self.make_constant(value)?;
                self.emit_constant_op(OpCode::Constant, constant);
            }

            Expression::Variable(name) => {
                self.compile_variable_get(name)?;
            }

            // ====================================================
            // FONCTION ANONYME / CALLBACK
            //
            // func(x) {
            //     return x * 2;
            // }
            //
            // Produit une Closure exactement comme une fonction
            // nommée, mais sans déclaration dans la table des globals
            // ou des locals.
            // ====================================================
            Expression::Function { params, body } => {
                let function = self.compile_function("", params, body)?;

                let function_constant =
                    self.make_constant(Value::new_function(std::rc::Rc::new(function.clone())))?;

                self.emit_closure(function_constant, &function.upvalues);
            }

            Expression::Binary {
                left,
                operator,
                right,
                line,
                column,
            } => match operator {
                BinaryOp::And => {
                    self.compile_logical_and(left, right)?;
                }

                BinaryOp::Or => {
                    self.compile_logical_or(left, right)?;
                }

                _ => {
                    self.compile_expression(left)?;
                    self.compile_expression(right)?;

                    // Positionne l'instruction de l'opérateur lui-même sur
                    // le début de l'opérande droit (ex. `age` dans
                    // `name + age`) plutôt que de laisser la position de
                    // l'instruction englobante : c'est ce qui permet au
                    // caret des diagnostics runtime (type mismatch...) de
                    // pointer sous l'opérande fautif plutôt que sous le
                    // début de la ligne. Voir `RuntimeError::NumericTypeError`.
                    self.current_line = *line;
                    self.current_column = *column;

                    self.compile_binary(operator.clone());
                }
            },

            Expression::Unary {
                operator,
                right,
                line,
                column,
            } => {
                self.compile_expression(right)?;

                self.current_line = *line;
                self.current_column = *column;

                match operator {
                    UnaryOp::Negate => self.emit_opcode(OpCode::Negate),
                    UnaryOp::Not => self.emit_opcode(OpCode::Not),
                    UnaryOp::BitNot => self.emit_opcode(OpCode::BitNot),
                }
            }

            Expression::Call {
                callee,
                arguments,
                line,
                column,
                ..
            } => {
                // La résolution sémantique a déjà été effectuée par le
                // TypeChecker. Le compilateur lit uniquement l'identité de
                // l'intrinsèque depuis `ResolvedCall` au lieu de redéduire
                // celle-ci à partir du nom du symbole.
                let resolved = self.resolved_call(*line, *column);
                let resolved_method_name = resolved.and_then(|resolved| match &resolved.target {
                    CallTarget::Method { name }
                    | CallTarget::StaticMethod { name } => Some(name.clone()),
                    _ => None,
                });
                let has_resolved_call = resolved.is_some();

                if let Expression::Member { object, name, .. } = callee.as_ref() {
                    if let Some(resolved_name) = resolved_method_name {
                        return self.compile_method_call(
                            object,
                            &resolved_name,
                            arguments,
                            *line,
                            *column,
                        );
                    }

                    // Les appels membres synthétiques créés par le compilateur
                    // (notamment les patterns `Some(x)`, `Ok(x)` et `Err(x)`)
                    // ne possèdent pas de position source enregistrée dans la
                    // table de résolution. Conserver le routage historique
                    // vers `InvokeMethod` lorsque la résolution est absente.
                    if !has_resolved_call {
                        return self.compile_method_call(
                            object,
                            name,
                            arguments,
                            *line,
                            *column,
                        );
                    }
                }

                let intrinsic = self
                    .resolved_call(*line, *column)
                    .and_then(ResolvedCall::intrinsic);

                if let Some(intrinsic) = intrinsic {
                    match intrinsic {
                        Intrinsic::Spawn => {
                            if arguments.len() > u8::MAX as usize {
                                return Err(CompileError::TooManyArguments);
                            }

                            for argument in arguments {
                                self.compile_expression(argument)?;
                            }

                            self.current_line = *line;
                            self.current_column = *column;
                            self.emit_bytes(OpCode::Spawn, (arguments.len() - 1) as u8);
                            return Ok(());
                        }

                        Intrinsic::Yield => {
                            self.current_line = *line;
                            self.current_column = *column;
                            self.emit_opcode(OpCode::Yield);
                            self.emit_opcode(OpCode::None);
                            return Ok(());
                        }

                        Intrinsic::Select => {
                            // `Select` attend toujours deux valeurs sur la
                            // pile : timeout (ou None), puis la liste des
                            // channels.
                            if let Some(timeout) = arguments.get(1) {
                                self.compile_expression(timeout)?;
                            } else {
                                self.emit_opcode(OpCode::None);
                            }

                            self.compile_expression(&arguments[0])?;
                            self.current_line = *line;
                            self.current_column = *column;
                            self.emit_opcode(OpCode::Select);
                            return Ok(());
                        }

                        Intrinsic::Sleep => {
                            self.compile_expression(&arguments[0])?;
                            self.current_line = *line;
                            self.current_column = *column;
                            self.emit_opcode(OpCode::Sleep);
                            self.emit_opcode(OpCode::None);
                            return Ok(());
                        }
                    }
                }

                self.compile_call(callee, arguments, *line, *column)?;
            }

            Expression::Array(elements) => {
                if elements.len() > u8::MAX as usize {
                    return Err(CompileError::TooManyArrayElements);
                }

                for element in elements {
                    self.compile_expression(element)?;
                }

                self.emit_bytes(OpCode::Array, elements.len() as u8);
            }

            Expression::Tuple(elements) => {
                if elements.len() > u8::MAX as usize {
                    return Err(CompileError::TooManyTupleElements);
                }

                for element in elements {
                    self.compile_expression(element)?;
                }

                self.emit_bytes(OpCode::Tuple, elements.len() as u8);
            }

            // Dict `{"name": v}` et Record `{ name: v }` : même schéma (couples
            // nom / valeur sur la pile), deux opcodes.
            Expression::Dict(fields) | Expression::Record(fields) => {
                if fields.len() > u8::MAX as usize {
                    return Err(CompileError::TooManyObjectFields);
                }

                for (key, value) in fields {
                    let key_constant = self.make_constant(Value::new_string(key.clone()))?;

                    self.emit_constant_op(OpCode::Constant, key_constant);

                    self.compile_expression(value)?;
                }

                let opcode = if matches!(expr, Expression::Record(_)) {
                    OpCode::Record
                } else {
                    OpCode::Object
                };

                self.emit_bytes(opcode, fields.len() as u8);
            }

            Expression::Index {
                object,
                index,
                line,
                column,
            } => {
                self.compile_expression(object)?;
                self.compile_expression(index)?;

                self.current_line = *line;
                self.current_column = *column;

                self.emit_opcode(OpCode::GetIndex);
            }

            Expression::Member {
                object,
                name,
                line,
                column,
            } => {
                // `x.length` n'est plus un cas particulier : la taille d'une
                // collection s'obtient par `x.size()` (méthode standard).
                self.compile_expression(object)?;

                let resolved_name = match self.resolved_member(*line, *column) {
                    Some(ResolvedMember::Property { name })
                    | Some(ResolvedMember::Dynamic { name }) => name.clone(),
                    None => name.clone(),
                };
                let name_constant = self.identifier_constant(&resolved_name)?;

                self.current_line = *line;
                self.current_column = *column;

                self.emit_constant_op(OpCode::GetProperty, name_constant);
            }

            Expression::Await(expression) => {
                self.compile_expression(expression)?;
                self.emit_opcode(OpCode::Await);
            }

            Expression::Try(expression) => {
                if !self.in_function {
                    return Err(CompileError::ReturnOutsidFunction);
                }

                self.compile_expression(expression)?;
                self.emit_opcode(OpCode::Try);

                // OP_TRY laisse [payload_ou_valeur_propagée, success].
                // En cas d'échec, la valeur propagée reste sur la pile et
                // les `finally` actifs sont exécutés avant `Return`.
                let failure_jump = self.emit_jump(OpCode::JumpIfFalse);

                // Succès : retirer le booléen, conserver le payload.
                self.emit_opcode(OpCode::Pop);
                let end_jump = self.emit_jump(OpCode::Jump);

                // Échec : retirer le booléen, conserver None/Err(...).
                self.patch_jump(failure_jump)?;
                self.emit_opcode(OpCode::Pop);
                self.emit_return_through_finally(true)?;

                self.patch_jump(end_jump)?;
            }

            Expression::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.compile_expression(condition)?;

                let else_jump = self.emit_jump(OpCode::JumpIfFalse);

                self.emit_opcode(OpCode::Pop);

                self.compile_expression(then_expr)?;

                let end_jump = self.emit_jump(OpCode::Jump);

                self.patch_jump(else_jump)?;

                self.emit_opcode(OpCode::Pop);

                self.compile_expression(else_expr)?;

                self.patch_jump(end_jump)?;
            }
            Expression::SelfValue => {
                self.compile_variable_get("self")?;
            }

            Expression::New {
                class_name,
                arguments,
                line,
                column,
                ..
            } => {
                if arguments.len() > u8::MAX as usize {
                    return Err(CompileError::TooManyArguments);
                }

                self.compile_variable_get(class_name)?;

                for argument in arguments {
                    self.compile_expression(argument)?;
                }

                self.current_line = *line;
                self.current_column = *column;

                self.emit_bytes(OpCode::NewInstance, arguments.len() as u8);
            }
        }

        Ok(())
    }
    // ============================================================
    //                       METHOD CALL
    // ============================================================

    pub(crate) fn compile_method_call(
        &mut self,
        object: &Expression,
        name: &str,
        arguments: &[Expression],
        line: usize,
        column: usize,
    ) -> Result<(), CompileError> {
        if arguments.len() > u8::MAX as usize {
            return Err(CompileError::TooManyArguments);
        }

        let method_constant = self.identifier_constant(name)?;

        self.compile_expression(object)?;

        for argument in arguments {
            self.compile_expression(argument)?;
        }

        self.current_line = line;
        self.current_column = column;

        self.emit_constant_op(OpCode::InvokeMethod, method_constant);
        self.emit_byte(arguments.len() as u8);

        Ok(())
    }

    // ============================================================
    //                           CALL
    // ============================================================

    pub(crate) fn compile_call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
        line: usize,
        column: usize,
    ) -> Result<(), CompileError> {
        if arguments.len() > u8::MAX as usize {
            return Err(CompileError::TooManyArguments);
        }

        self.compile_expression(callee)?;

        for argument in arguments {
            self.compile_expression(argument)?;
        }

        self.current_line = line;
        self.current_column = column;

        self.emit_bytes(OpCode::Call, arguments.len() as u8);

        Ok(())
    }

    // ============================================================
    //                     LOGICAL OPERATORS
    // ============================================================

    pub(crate) fn compile_logical_and(
        &mut self,
        left: &Expression,
        right: &Expression,
    ) -> Result<(), CompileError> {
        self.compile_expression(left)?;

        let end_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_opcode(OpCode::Pop);

        self.compile_expression(right)?;

        self.patch_jump(end_jump)?;

        Ok(())
    }

    pub(crate) fn compile_logical_or(
        &mut self,
        left: &Expression,
        right: &Expression,
    ) -> Result<(), CompileError> {
        self.compile_expression(left)?;

        let end_jump = self.emit_jump(OpCode::JumpIfFalse);

        let right_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(end_jump)?;

        self.emit_opcode(OpCode::Pop);

        self.compile_expression(right)?;

        self.patch_jump(right_jump)?;

        Ok(())
    }

    // ============================================================
    //                     BINARY OPERATORS
    // ============================================================

    pub(crate) fn compile_binary(&mut self, operator: BinaryOp) {
        let opcode = match operator {
            BinaryOp::Add => OpCode::Add,
            BinaryOp::Subtract => OpCode::Subtract,
            BinaryOp::Multiply => OpCode::Multiply,
            BinaryOp::Divide => OpCode::Divide,
            BinaryOp::Modulo => OpCode::Modulo,

            BinaryOp::Equal => OpCode::Equal,

            BinaryOp::NotEqual => {
                self.emit_opcode(OpCode::Equal);
                self.emit_opcode(OpCode::Not);
                return;
            }

            BinaryOp::Less => OpCode::Less,

            // `<=` / `>=` sont des opcodes à part entière : les compiler en
            // `!(a > b)` / `!(a < b)` donnait `NaN <= x == true`.
            BinaryOp::LessEqual => OpCode::LessEqual,

            BinaryOp::Greater => OpCode::Greater,

            BinaryOp::GreaterEqual => OpCode::GreaterEqual,
            BinaryOp::Is => OpCode::Is,
            BinaryOp::BitAnd => OpCode::BitAnd,
            BinaryOp::BitOr => OpCode::BitOr,
            BinaryOp::BitXor => OpCode::BitXor,
            BinaryOp::ShiftLeft => OpCode::ShiftLeft,
            BinaryOp::ShiftRight => OpCode::ShiftRight,

            _ => unreachable!(),
        };

        self.emit_opcode(opcode);
    }
}
