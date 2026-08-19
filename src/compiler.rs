use std::{collections::HashMap, fmt::Display};

use crate::{
    ast::{BinaryOp, Expression, Literal, Statement, UnaryOp},
    chunk::{Chunk, OpCode},
    value::Value::{self, Boolean, Nil, Number},
};
// #[allow(dead_code)]
pub struct Local {
    pub name: String,
    pub depth: Option<usize>,
}

enum VariableLocation {
    Local(usize),
    Global(usize),
}

#[derive(Debug)]
// #[allow(dead_code)]
pub enum CompileError {
    VariableAlreadyDeclared(String),
    VariableUseInInitializer(String),
    UndefinedVariable(String),
    TooManyConstants,
    TooManyLocals,
}

impl Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::VariableAlreadyDeclared(e) => write!(f, "Variable {e} déja déclarer"),
            CompileError::VariableUseInInitializer(e) => write!(f, "Variable {e} non initialisé"),
            CompileError::UndefinedVariable(e) => write!(f, "Variable {e} non définie"),
            CompileError::TooManyConstants => write!(f, "Trops de constant"),
            CompileError::TooManyLocals => write!(f, "Trops de local"),
        }
    }
}
// #[allow(dead_code)]
pub struct Compiler {
    pub globals: HashMap<String, u8>,
    pub chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: usize,
}
// #[allow(dead_code)]
impl Compiler {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

    pub fn compile(mut self, statements: &[Statement], line: usize) -> Chunk {
        for statement in statements {
            match self.compile_statement(statement, line) {
                Ok(success) => success,
                Err(error) => eprintln!("{}", error),
            }
        }
        self.emit_opcode(OpCode::Return, line);
        self.chunk
    }

    fn emit_byte(&mut self, byte: u8, line: usize) {
        self.chunk.write(byte, line);
    }

    fn emit_opcode(&mut self, opcode: OpCode, line: usize) {
        self.emit_byte(opcode.into(), line);
    }

    fn emit_bytes(&mut self, opcode: OpCode, operand: u8, line: usize) {
        self.emit_opcode(opcode, line);
        self.emit_byte(operand, line);
    }

    fn make_constant(&mut self, value: Value) -> Result<u8, CompileError> {
        let index = self.chunk.add_constant(value);
        if index > u8::MAX as usize {
            return Err(CompileError::TooManyConstants);
        }

        Ok(index as u8)
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self, line: usize) {
        self.scope_depth -= 1;

        while let Some(local) = self.locals.last() {
            if local.depth < Some(self.scope_depth) {
                break;
            }

            self.emit_opcode(OpCode::Pop, line);

            self.locals.pop();
        }
    }

    fn resolve_local(&self, name: &str) -> Result<Option<usize>, CompileError> {
        for (index, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                if local.depth.is_none() {
                    return Err(CompileError::VariableUseInInitializer(name.to_string()));
                }

                return Ok(Some(index));
            }
        }
        Ok(None)
    }

    fn resolve_variable(&self, name: &str) -> Result<VariableLocation, CompileError> {
        if let Some(slot) = self.resolve_local(name)? {
            return Ok(VariableLocation::Local(slot));
        }
        if let Some(global) = self.globals.get(name) {
            return Ok(VariableLocation::Global((*global).into()));
        }

        Err(CompileError::UndefinedVariable(name.to_string()))
    }

    fn declare_local(&mut self, name: &str) -> Result<(), CompileError> {
        if self.scope_depth == 0 {
            return Ok(());
        }

        //Empèche la rédeclaration dans la même scope
        for local in self.locals.iter().rev() {
            if let Some(depth) = local.depth {
                if depth < self.scope_depth {
                    break;
                }
                if local.name == name {
                    return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
                }
            }
        }
        if self.locals.len() >= u8::MAX as usize {
            return Err(CompileError::TooManyLocals);
        }

        self.locals.push(Local {
            name: name.to_string(),
            depth: Some(self.scope_depth),
        });

        Ok(())
    }

    fn compile_global_var(
        &mut self,
        name: &str,
        initializer: Option<&Expression>,
        line: usize,
    ) -> Result<(), CompileError> {
        if self.globals.contains_key(name) {
            return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
        }

        let name_constant = self.identifier_constant(name)?;

        match initializer {
            Some(expr) => {
                self.compile_expression(expr, line)?;
            }

            None => {
                self.emit_opcode(OpCode::Nil, line);
            }
        }

        self.emit_bytes(OpCode::DefineGlobal, name_constant, line);
        self.globals.insert(name.to_string(), name_constant);

        Ok(())
    }

    fn mark_initialized(&mut self) {
        if self.scope_depth == 0 {
            return;
        }
        if let Some(local) = self.locals.last_mut() {
            let _ = local.depth == Some(self.scope_depth);
        }
    }

    fn identifier_constant(&mut self, name: &str) -> Result<u8, CompileError> {
        self.make_constant(Value::String(name.to_string()))
    }

    fn compile_var(
        &mut self,
        name: &str,
        initializer: Option<&Expression>,
        line: usize,
    ) -> Result<(), CompileError> {
        //LOCAL
        if self.scope_depth > 0 {
            self.declare_local(name)?;

            match initializer {
                Some(expr) => {
                    self.compile_expression(expr, line)?;
                }
                None => {
                    self.emit_opcode(OpCode::Nil, line);
                }
            }
            self.mark_initialized();

            return Ok(());
        }

        //GLOBAL
        self.compile_global_var(name, initializer, line)
    }
    #[allow(unused_variables)]
    fn compile_expression(&mut self, expr: &Expression, line: usize) -> Result<(), CompileError> {
        match expr {
            Expression::Literal(value) => match value {
                Literal::Number(v) => {
                    let constant = self.make_constant(Number(*v))?;
                    self.emit_bytes(OpCode::Constant, constant, line);
                }
                Literal::String(v) => {
                    let constant = self.make_constant(Value::String(v.clone()))?;
                    self.emit_bytes(OpCode::Constant, constant, line);
                }
                Literal::Bool(v) => {
                    let constant = self.make_constant(Boolean(*v))?;
                    self.emit_bytes(OpCode::Constant, constant, line);
                }
                Literal::Nil => {
                    let constant = self.make_constant(Nil)?;
                    self.emit_bytes(OpCode::Constant, constant, line);
                }
            },

            Expression::Variable(name) => {
                self.compile_variable_get(name, line)?;
            }

            Expression::Assignment { name, value } => {
                self.compile_expression(value, line)?;
                self.compile_variable_set(name, line)?;
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                self.compile_expression(left, line)?;
                self.compile_expression(right, line)?;
                self.compile_binary(operator.clone(), line);
            }
            Expression::Unary { operator, right } => {
                self.compile_expression(right, line)?;
                match operator {
                    UnaryOp::Negate => self.emit_opcode(OpCode::Negate, line),
                    UnaryOp::Not => self.emit_opcode(OpCode::Not, line),
                }
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    fn compile_variable_get(&mut self, name: &str, line: usize) -> Result<(), CompileError> {
        match self.resolve_variable(name)? {
            VariableLocation::Local(slot) => {
                self.emit_bytes(OpCode::GetLocal, slot as u8, line);
            }
            VariableLocation::Global(index) => {
                self.emit_bytes(OpCode::GetGlobal, index as u8, line);
            }
        }

        Ok(())
    }

    fn compile_variable_set(&mut self, name: &str, line: usize) -> Result<(), CompileError> {
        match self.resolve_variable(name)? {
            VariableLocation::Local(slot) => {
                self.emit_bytes(OpCode::SetLocal, slot as u8, line);
            }
            VariableLocation::Global(index) => {
                self.emit_bytes(OpCode::SetGlobal, index as u8, line);
            }
        }

        Ok(())
    }

    fn compile_binary(&mut self, operator: crate::ast::BinaryOp, line: usize) {
        match operator {
            BinaryOp::Add => {
                self.emit_opcode(OpCode::Add, line);
            }
            BinaryOp::Subtract => {
                self.emit_opcode(OpCode::Subtract, line);
            }
            BinaryOp::Multiply => {
                self.emit_opcode(OpCode::Multiply, line);
            }
            BinaryOp::Divide => {
                self.emit_opcode(OpCode::Divide, line);
            }
            BinaryOp::Modulo => {
                self.emit_opcode(OpCode::Modulo, line);
            }
            BinaryOp::Equal => {
                self.emit_opcode(OpCode::Equal, line);
            }
            BinaryOp::NotEqual => {
                self.emit_opcode(OpCode::Equal, line);
                self.emit_opcode(OpCode::Not, line);
            }
            BinaryOp::Less => {
                self.emit_opcode(OpCode::Less, line);
            }
            BinaryOp::LessEqual => {
                self.emit_opcode(OpCode::Greater, line);
                self.emit_opcode(OpCode::Not, line);
            }
            BinaryOp::Greater => {
                self.emit_opcode(OpCode::Greater, line);
            }
            BinaryOp::GreaterEqual => {
                self.emit_opcode(OpCode::Less, line);
                self.emit_opcode(OpCode::Not, line);
            }

            _ => unreachable!(),
        }
    }
    #[allow(unused_variables)]
    pub fn compile_statement(&mut self, stmt: &Statement, line: usize) -> Result<(), CompileError> {
        match stmt {
            Statement::Expression { expression } => {
                self.compile_expression(expression, line)?;
            }

            Statement::Let { name, value } => {
                self.compile_var(name, Some(value), line)?;
            }

            Statement::Block(statements) => {
                self.begin_scope();

                for statement in statements {
                    self.compile_statement(statement, line)?;
                }
                self.end_scope(line);
            }
            Statement::Assignment { name, value } => {
                self.compile_expression(value, line)?;
                self.compile_variable_set(name, line)?;
            }
            Statement::Print(expression) => {
                self.compile_expression(expression, line)?;
                self.emit_opcode(OpCode::Print, line);
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => todo!(),
        }

        Ok(())
    }
}
