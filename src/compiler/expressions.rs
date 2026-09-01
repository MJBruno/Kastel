use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use crate::frontend::ast::*;
use crate::runtime::value::Value;

use super::compiler::Compiler;

impl Compiler {
    // ============================================================
    //                      EXPRESSION
    // ============================================================
    #[allow(unused_variables)]
    pub(crate) fn compile_expression(&mut self, expr: &Expression) -> Result<(), CompileError> {
        match expr {
            Expression::Literal(value) => {
                let value = match value {
                    Literal::Number(v) => Value::Number(*v),

                    Literal::String(v) => Value::String(v.clone()),

                    Literal::Bool(v) => Value::Boolean(*v),

                    Literal::Nil => Value::Nil,
                };

                let constant = self.make_constant(value)?;

                self.emit_bytes(OpCode::Constant, constant);
            }

            Expression::Variable(name) => {
                self.compile_variable_get(name)?;
            }

            Expression::Binary {
                left,
                operator,
                right,
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

                    self.compile_binary(operator.clone());
                }
            },

            Expression::Unary { operator, right } => {
                self.compile_expression(right)?;

                match operator {
                    UnaryOp::Negate => self.emit_opcode(OpCode::Negate),

                    UnaryOp::Not => self.emit_opcode(OpCode::Not),
                }
            }

            Expression::Call { callee, arguments } => {
                if let Expression::Member { object, name } = callee.as_ref() {
                    match name.as_str() {
                        "push" | "pop" | "insert" | "remove" | "clear" | "contains" => {
                            return self.compile_array_method_call(object, name, arguments);
                        }

                        _ => {}
                    }
                }

                self.compile_call(callee, arguments)?;
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

            Expression::Object(fields) => {
                if fields.len() > u8::MAX as usize {
                    return Err(CompileError::TooManyArrayElements);
                }

                for (key, value) in fields {
                    // La clé est poussée comme une constante String, au
                    // même titre qu'une expression normale — même schéma
                    // que le tableau (push N valeurs, puis un opcode qui
                    // les consomme toutes), mais en alternant clé/valeur.
                    let key_constant = self.make_constant(Value::String(key.clone()))?;

                    self.emit_bytes(OpCode::Constant, key_constant);

                    self.compile_expression(value)?;
                }

                self.emit_bytes(OpCode::Object, fields.len() as u8);
            }

            Expression::Index { object, index } => {
                self.compile_expression(object)?;
                self.compile_expression(index)?;

                self.emit_opcode(OpCode::GetIndex);
            }

            Expression::Member { object, name } => {
                if name == "length" {
                    self.compile_array_member(object, name)?;
                } else {
                    self.compile_expression(object)?;

                    let name_constant = self.identifier_constant(name)?;

                    self.emit_bytes(OpCode::GetProperty, name_constant);
                }
            }

            Expression::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.compile_expression(condition)?;

                let else_jump = self.emit_jump(OpCode::JumpIfFalse);
                self.emit_opcode(OpCode::Pop); // dépile la condition (chemin "vrai")

                self.compile_expression(then_expr)?;

                let end_jump = self.emit_jump(OpCode::Jump);

                self.patch_jump(else_jump);
                self.emit_opcode(OpCode::Pop); // dépile la condition (chemin "faux")

                self.compile_expression(else_expr)?;

                self.patch_jump(end_jump);
            }
        }

        Ok(())
    }

    pub(crate) fn compile_array_method_call(
        &mut self,
        object: &Expression,
        name: &str,
        arguments: &[Expression],
    ) -> Result<(), CompileError> {
        match name {
            "push" => {
                if arguments.len() != 1 {
                    return Err(CompileError::WrongArgumentCount {
                        expected: 1,
                        found: arguments.len(),
                    });
                }

                self.compile_expression(object)?;

                self.compile_expression(&arguments[0])?;

                self.emit_opcode(OpCode::ArrayPush);

                Ok(())
            }

            "pop" => {
                if !arguments.is_empty() {
                    return Err(CompileError::WrongArgumentCount {
                        expected: 0,
                        found: arguments.len(),
                    });
                }

                self.compile_expression(object)?;

                self.emit_opcode(OpCode::ArrayPop);

                Ok(())
            }

            "insert" => {
                if arguments.len() != 2 {
                    return Err(CompileError::WrongArgumentCount {
                        expected: 2,
                        found: arguments.len(),
                    });
                }

                self.compile_expression(object)?;

                self.compile_expression(&arguments[0])?;

                self.compile_expression(&arguments[1])?;

                self.emit_opcode(OpCode::ArrayInsert);

                Ok(())
            }

            "remove" => {
                if arguments.len() != 1 {
                    return Err(CompileError::WrongArgumentCount {
                        expected: 1,
                        found: arguments.len(),
                    });
                }

                self.compile_expression(object)?;

                self.compile_expression(&arguments[0])?;

                self.emit_opcode(OpCode::ArrayRemove);

                Ok(())
            }
            "clear" => {
                if !arguments.is_empty() {
                    return Err(CompileError::WrongArgumentCount {
                        expected: 0,
                        found: arguments.len(),
                    });
                }

                self.compile_expression(object)?;

                self.emit_opcode(OpCode::ArrayClear);

                Ok(())
            }

            "contains" => {
                if arguments.len() != 1 {
                    return Err(CompileError::WrongArgumentCount {
                        expected: 1,
                        found: arguments.len(),
                    });
                }

                self.compile_expression(object)?;

                self.compile_expression(&arguments[0])?;

                self.emit_opcode(OpCode::ArrayContains);

                Ok(())
            }

            _ => Err(CompileError::InvalidMemberAccess {
                name: name.to_string(),
            }),
        }
    }

    pub(crate) fn compile_array_member(
        &mut self,
        object: &Expression,
        name: &str,
    ) -> Result<(), CompileError> {
        match name {
            "length" => {
                self.compile_expression(object)?;

                self.emit_opcode(OpCode::ArrayLength);

                Ok(())
            }

            "push" | "pop" => Err(CompileError::InvalidMemberAccess {
                name: name.to_string(),
            }),

            _ => Err(CompileError::InvalidMemberAccess {
                name: name.to_string(),
            }),
        }
    }

    pub(crate) fn compile_call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
    ) -> Result<(), CompileError> {
        self.compile_expression(callee)?;

        for argument in arguments {
            self.compile_expression(argument)?;
        }

        if arguments.len() > u8::MAX as usize {
            return Err(CompileError::TooManyArguments);
        }

        self.emit_bytes(OpCode::Call, arguments.len() as u8);

        Ok(())
    }

    pub(crate) fn compile_logical_and(
        &mut self,
        left: &Expression,
        right: &Expression,
    ) -> Result<(), CompileError> {
        self.compile_expression(left)?;

        let end_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_opcode(OpCode::Pop);

        self.compile_expression(right)?;

        self.patch_jump(end_jump);

        Ok(())
    }

    pub(crate) fn compile_logical_or(
        &mut self,
        left: &Expression,
        right: &Expression,
    ) -> Result<(), CompileError> {
        self.compile_expression(left)?;

        self.emit_opcode(OpCode::Not);

        let end_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_opcode(OpCode::Not);

        self.emit_opcode(OpCode::Pop);

        self.compile_expression(right)?;

        self.patch_jump(end_jump);

        Ok(())
    }

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

            _ => unreachable!(),
        };

        self.emit_opcode(opcode);
    }
}