use std::{collections::HashMap, fmt::Display};

use crate::{
    ast::{BinaryOp, Expression, Literal, Statement, UnaryOp},
    chunk::{Chunk, OpCode},
    value::Value,
};

pub struct Local {
    pub name: String,
    pub depth: Option<usize>,
    //Répresente directement la position du variable local dans la pile
    pub slot: u8,
}

pub struct LocalTable {
    locals: Vec<Local>,
}
#[allow(dead_code)]
impl LocalTable {
    pub fn new() -> Self {
        Self { locals: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.locals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.locals.is_empty()
    }
    fn declare_local(&mut self, name: &str, depth: usize) -> Result<u8, CompileError> {
        //Empèche la rédeclaration dans la même scope
        for local in self.locals.iter().rev() {
            if let Some(local_depth) = local.depth {
                if local_depth < depth {
                    break;
                }
                if local.name == name {
                    return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
                }
            }
        }

        let slot = self.locals.len();
        if self.locals.len() >= u8::MAX as usize {
            return Err(CompileError::TooManyLocals);
        }

        self.locals.push(Local {
            name: name.to_string(),
            depth: Some(depth),
            slot: slot as u8,
        });

        Ok(slot as u8)
    }

    fn mark_initialized(&mut self, depth: usize) {
        if let Some(local) = self.locals.last_mut() {
            let _ = local.depth == Some(depth);
        }
    }

    fn resolve_local(&self, name: &str) -> Result<Option<u8>, CompileError> {
        for local in self.locals.iter().rev() {
            if local.name != name {
                continue;
            }
            if local.depth.is_none() {
                return Err(CompileError::VariableUseInInitializer(name.to_string()));
            }
            return Ok(Some(local.slot));
        }
        Ok(None)
    }

    pub fn pop_scope(&mut self, depth: usize) -> usize {
        let mut count = 0;
        while let Some(local) = self.locals.last() {
            let local_depth = match local.depth {
                Some(depth) => depth,
                None => break,
            };

            if local_depth <= depth {
                break;
            }

            self.locals.pop();
            count += 1;
        }
        count
    }
}

enum VariableLocation {
    Local(usize),
    Global(usize),
}

#[derive(Debug)]
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
            CompileError::TooManyConstants => write!(f, "Trop de constant"),
            CompileError::TooManyLocals => write!(f, "Trop de local"),
        }
    }
}

pub struct Compiler {
    pub globals: HashMap<String, u8>,
    pub chunk: Chunk,
    locals: LocalTable,
    pub scope_depth: usize,
}

#[allow(dead_code)]
impl Compiler {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            chunk: Chunk::new(),
            locals: LocalTable::new(),
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

        let count = self.locals.pop_scope(self.scope_depth);

        for _ in 0..count {
            self.emit_opcode(OpCode::Pop, line);
        }
    }

    fn resolve_variable(&self, name: &str) -> Result<VariableLocation, CompileError> {
        if let Some(slot) = self.locals.resolve_local(name)? {
            return Ok(VariableLocation::Local(slot.into()));
        }
        if let Some(index) = self.globals.get(name) {
            return Ok(VariableLocation::Global((*index).into()));
        }

        Err(CompileError::UndefinedVariable(name.to_string()))
    }

    fn compile_var(
        &mut self,
        name: &str,
        initializer: Option<&Expression>,
        line: usize,
    ) -> Result<(), CompileError> {
        if self.scope_depth > 0 {
            return self.compile_local_var(name, initializer, line);
        }

        self.compile_global_var(name, initializer, line)
    }

    fn identifier_constant(&mut self, name: &str) -> Result<u8, CompileError> {
        self.make_constant(Value::String(name.to_string()))
    }

    fn compile_local_var(
        &mut self,
        name: &str,
        initializer: Option<&Expression>,
        line: usize,
    ) -> Result<(), CompileError> {
        let slot = self.locals.declare_local(name, self.scope_depth)?;

        match initializer {
            Some(expr) => self.compile_expression(expr, line)?,
            None => {
                Ok(self.emit_opcode(OpCode::Nil, line))?;
            }
        }
        self.locals.mark_initialized(self.scope_depth);

        debug_assert_eq!(self.locals.len() - 1, slot as usize);

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
            Some(expr) => self.compile_expression(expr, line)?,
            None => {
                Ok(self.emit_opcode(OpCode::Nil, line))?;
            }
        }

        self.emit_bytes(OpCode::DefineGlobal, name_constant, line);

        self.globals.insert(name.to_string(), name_constant);

        Ok(())
    }

    fn compile_get(&mut self, name: &str, line: usize) -> Result<(), CompileError> {
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
    fn compile_set(&mut self, name: &str, line: usize) -> Result<(), CompileError> {
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

    fn compile_expression(&mut self, expr: &Expression, line: usize) -> Result<(), CompileError> {
        match expr {
            Expression::Literal(value) => match value {
                Literal::Number(v) => {
                    let constant = self.make_constant(Value::Number(*v))?;
                    self.emit_bytes(OpCode::Constant, constant, line);
                }
                Literal::String(v) => {
                    let constant = self.make_constant(Value::String(v.clone()))?;
                    self.emit_bytes(OpCode::Constant, constant, line);
                }
                Literal::Bool(v) => {
                    let constant = self.make_constant(Value::Boolean(*v))?;
                    self.emit_bytes(OpCode::Constant, constant, line);
                }
                Literal::Nil => {
                    let constant = self.make_constant(Value::Nil)?;
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

    fn compile_binary(&mut self, operator: BinaryOp, line: usize) {
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
            } => match self.compile_if(condition, then_branch, else_branch.as_ref(), line) {
                Ok(execute_if) => execute_if,
                Err(error) => eprintln!("{error}"),
            },
        }

        Ok(())
    }

    fn emit_jump(&mut self, opcode: OpCode, line: usize) -> usize {
        self.emit_opcode(opcode, line);

        //Deux octets réservés pour l'adresse
        self.emit_byte(0xff, line);
        self.emit_byte(0xff, line);

        self.chunk.code.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.code.len() - offset - 2;
        assert!(jump <= u16::MAX as usize, "Jump trop grand");

        let jump = jump as u16;

        self.chunk.code[offset] = (jump >> 8) as u8;
        self.chunk.code[offset + 1] = (jump & 0xff) as u8;
    }

    fn compile_if(
        &mut self,
        condition: &Expression,
        then_branch: &[Statement],
        else_branch: Option<&Vec<Statement>>,
        line: usize,
    ) -> Result<(), CompileError> {
        //condition
        self.compile_expression(condition, line)?;

        //Saut vers ELSE ou FIN
        let then_jump = self.emit_jump(OpCode::JumpIfFalse, line);

        for statement in then_branch {
            self.compile_statement(statement, line)?;
        }

        //ELSE présent ?
        if let Some(else_branch) = else_branch {
            //Sauter le ELSE après avoir éxécuté THEN
            let else_jump = self.emit_jump(OpCode::Jump, line);

            //déstination du JumpIfFalse
            self.patch_jump(then_jump);

            //Rétiré la condition false
            self.emit_opcode(OpCode::Pop, line);

            for statement in else_branch {
                self.compile_statement(statement, line)?;
            }

            //Déstination final
            self.patch_jump(else_jump);
        } else {
            //Déstination final
            self.patch_jump(then_jump);

            //Rétire la condition false
            self.emit_opcode(OpCode::Pop, line);
        }
        Ok(())
    }
}

// --------------------------------------------------
//                  FONCTION
// --------------------------------------------------
