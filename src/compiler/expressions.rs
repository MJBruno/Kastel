use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use crate::frontend::ast::*;
use crate::runtime::value::Value;

use super::compiler::{Compiler, MAX_EXPRESSION_DEPTH};

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
            // function(x) {
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
                if let Expression::Member { object, name, .. } = callee.as_ref() {
                    if matches!(object.as_ref(), Expression::Base) {
                        return self.compile_base_method_call(name, arguments, *line, *column);
                    }

                    return self.compile_method_call(object, name, arguments, *line, *column);
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

                let name_constant = self.identifier_constant(name)?;

                self.current_line = *line;
                self.current_column = *column;

                self.emit_constant_op(OpCode::GetProperty, name_constant);
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
            Expression::Base => {
                return Err(CompileError::InternalCompilerError(
                    "'base' doit être utilisé pour appeler une méthode".to_string(),
                ));
            }
            Expression::This => {
                self.compile_variable_get("this")?;
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
    pub(crate) fn compile_base_method_call(
        &mut self,
        name: &str,
        arguments: &[Expression],
        line: usize,
        column: usize,
    ) -> Result<(), CompileError> {
        if arguments.len() > u8::MAX as usize {
            return Err(CompileError::TooManyArguments);
        }

        let method_constant = self.identifier_constant(name)?;

        // base.method(...) utilise la même instance que la méthode courante.
        self.compile_expression(&Expression::This)?;

        for argument in arguments {
            self.compile_expression(argument)?;
        }

        self.current_line = line;
        self.current_column = column;

        self.emit_constant_op(OpCode::InvokeBaseMethod, method_constant);
        self.emit_byte(arguments.len() as u8);

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

            BinaryOp::LessEqual => {
                self.emit_opcode(OpCode::Greater);
                self.emit_opcode(OpCode::Not);
                return;
            }

            BinaryOp::Greater => OpCode::Greater,

            BinaryOp::GreaterEqual => {
                self.emit_opcode(OpCode::Less);
                self.emit_opcode(OpCode::Not);
                return;
            }
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
