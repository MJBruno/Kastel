use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use crate::ast::{BinaryOp, Expression, Literal, Statement, UnaryOp};
use crate::chunk::{Chunk, OpCode};
use crate::error::CompileError;
use crate::value::Value;

pub struct Local {
    pub name: String,
    pub depth: Option<usize>,
    //Répresente directement la position du variable local dans la pile
    pub slot: u8,
}
enum VariableLocation {
    Local(usize),
    Global,
}
pub struct LocalTable {
    locals: Vec<Local>,
}

impl LocalTable {
    pub fn new() -> Self {
        Self { locals: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.locals.len()
    }

    fn remove_last(&mut self) {
        self.locals.pop();
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
            debug_assert_eq!(local.depth, Some(depth));
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

//Pour géré la boucle avec break/continue
struct LoopContext {
    continue_target: usize,
    break_jumps: Vec<usize>,
    scope_depth: usize,
}
#[allow(dead_code)]
pub struct Compiler {
    globals: Rc<RefCell<HashSet<String>>>,
    chunk: Chunk,
    locals: LocalTable,
    scope_depth: usize,
    loops: Vec<LoopContext>,

    function_name: Option<String>,
    function_arity: u8,
    in_function: bool,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            globals: Rc::new(RefCell::new(HashSet::new())),
            chunk: Chunk::new(),
            locals: LocalTable::new(),
            scope_depth: 0,
            loops: Vec::new(),
            function_name: None,
            function_arity: 0,
            in_function: false,
        }
    }

    pub fn new_function(name: String, globals: Rc<RefCell<HashSet<String>>>) -> Self {
        Self {
            globals,
            chunk: Chunk::new(),
            locals: LocalTable::new(),
            scope_depth: 0,
            loops: Vec::new(),
            function_name: Some(name),
            function_arity: 0,
            in_function: true,
        }
    }

    fn add_parametre(&mut self, name: &str) -> Result<(), CompileError> {
        let slot = self.locals.declare_local(name, self.scope_depth)?;
        self.locals.mark_initialized(self.scope_depth);

        debug_assert_eq!(slot, self.function_arity);

        self.function_arity += 1;

        Ok(())
    }

    pub fn compile(
        mut self,
        statements: &[Statement],
        line: usize,
    ) -> Result<Function, CompileError> {
        for statement in statements {
            self.compile_statement(statement, line)?;
        }

        self.emit_opcode(OpCode::Halt, line);

        Ok(Function {
            name: "<script>".to_string(),
            arity: 0,
            chunk: self.chunk,
        })
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

    // --------------------------------------------------
    //                  COMPILER_EXPRESSION
    // --------------------------------------------------
    #[allow(dead_code)]
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

            Expression::Call { callee, arguments } => {
                self.compile_call(callee, arguments, line)?;
            }
        }

        Ok(())
    }

    // --------------------------------------------------
    //                  COMPILER_VARIABLE: GET/SET
    // --------------------------------------------------

    fn resolve_variable(&self, name: &str) -> Result<VariableLocation, CompileError> {
        if let Some(slot) = self.locals.resolve_local(name)? {
            return Ok(VariableLocation::Local(slot as usize));
        }

        if self.globals.borrow().contains(name) {
            return Ok(VariableLocation::Global);
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

    fn emit_scope_cleanup(&mut self, target_depth: usize, line: usize) {
        while let Some(local) = self.locals.locals.last() {
            let depth = match local.depth {
                Some(depth) => depth,
                None => break,
            };

            if depth <= target_depth {
                break;
            }
            self.emit_opcode(OpCode::Pop, line);

            self.locals.remove_last();
        }
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
        if self.globals.borrow().contains(name) {
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

        self.globals.borrow_mut().insert(name.to_string());

        Ok(())
    }

    fn compile_variable_get(&mut self, name: &str, line: usize) -> Result<(), CompileError> {
        match self.resolve_variable(name)? {
            VariableLocation::Local(slot) => {
                self.emit_bytes(OpCode::GetLocal, slot as u8, line);
            }

            VariableLocation::Global => {
                let name_constant = self.identifier_constant(name)?;

                self.emit_bytes(OpCode::GetGlobal, name_constant, line);
            }
        }

        Ok(())
    }

    fn compile_variable_set(&mut self, name: &str, line: usize) -> Result<(), CompileError> {
        match self.resolve_variable(name)? {
            VariableLocation::Local(slot) => {
                self.emit_bytes(OpCode::SetLocal, slot as u8, line);
            }

            VariableLocation::Global => {
                let name_constant = self.identifier_constant(name)?;

                self.emit_bytes(OpCode::SetGlobal, name_constant, line);
            }
        }

        Ok(())
    }

    // --------------------------------------------------
    //                  COMPILER_BINARY
    // --------------------------------------------------

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

    // --------------------------------------------------
    //                  COMPILER_STATEMENT
    // --------------------------------------------------

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

            Statement::While { condition, body } => {
                match self.compile_while(condition, body, line) {
                    Ok(execute_while) => execute_while,
                    Err(error) => eprintln!("{error}"),
                }
            }
            Statement::Function { name, params, body } => {
                self.compile_function_statement(name, params, body, line)?;
            }
            Statement::Break => self.compile_break(line)?,
            Statement::Continue => self.compile_continue(line)?,
            Statement::Return { value } => {
                self.compile_return(value.as_ref(), line)?;
            }
        }

        Ok(())
    }

    fn emit_loop(&mut self, loop_start: usize, line: usize) {
        self.emit_opcode(OpCode::Loop, line);
        let offset = self.chunk.code.len() + 2 - loop_start;
        assert!(offset <= u16::MAX as usize, "Loop body too large");
        let offset = offset as u16;
        self.emit_byte((offset >> 8) as u8, line);
        self.emit_byte((offset & 0xff) as u8, line);
    }

    // --------------------------------------------------
    //                  COMPILER_IF/ELSE
    // --------------------------------------------------

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

    // --------------------------------------------------
    //                  COMPILER_WHILE
    // --------------------------------------------------

    fn compile_while(
        &mut self,
        condition: &Expression,
        body: &[Statement],
        line: usize,
    ) -> Result<(), CompileError> {
        //Début de la condition
        let loop_start = self.chunk.code.len();

        //La condition est égale
        //La destination de continue
        let continue_target = loop_start;

        //CONDITION
        self.compile_expression(condition, line)?;

        //si c'est faux -> sortie
        let exit_jump = self.emit_jump(OpCode::JumpIfFalse, line);

        //La condition est vraie
        self.emit_opcode(OpCode::Pop, line);

        self.loops.push(LoopContext {
            continue_target,
            break_jumps: Vec::new(),
            scope_depth: self.scope_depth,
        });

        //BODY
        self.begin_scope();

        for statement in body {
            self.compile_statement(statement, line)?;
        }

        self.end_scope(line);

        //Rétour au début
        self.emit_loop(loop_start, line);

        //EXIT
        self.patch_jump(exit_jump);

        //condition false
        self.emit_opcode(OpCode::Pop, line);

        //Récupèrer le contexte
        let loop_context = self.loops.pop().expect("loop stack underflow");

        //Patch de tout les break
        for break_jump in loop_context.break_jumps {
            self.patch_jump(break_jump);
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

    // --------------------------------------------------
    //                  COMPILER_BREAK
    // --------------------------------------------------

    fn compile_break(&mut self, line: usize) -> Result<(), CompileError> {
        let loop_depth = match self.loops.last() {
            Some(loop_context) => loop_context.scope_depth,

            None => {
                return Err(CompileError::BreakOutsideLoop);
            }
        };

        self.emit_scope_cleanup(loop_depth, line);

        let jump = self.emit_jump(OpCode::Jump, line);

        self.loops.last_mut().unwrap().break_jumps.push(jump);

        Ok(())
    }

    // --------------------------------------------------
    //                  COMPILER_CONTINUE
    // --------------------------------------------------

    fn compile_continue(&mut self, line: usize) -> Result<(), CompileError> {
        let (continue_target, loop_depth) = match self.loops.last() {
            Some(loop_context) => (loop_context.continue_target, loop_context.scope_depth),

            None => {
                return Err(CompileError::ContinueOutsideLoop);
            }
        };

        self.emit_scope_cleanup(loop_depth, line);

        self.emit_loop(continue_target, line);

        Ok(())
    }

    // --------------------------------------------------
    //                  COMPILER_FUNCTION
    // --------------------------------------------------

    fn compile_function_statement(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Statement],
        line: usize,
    ) -> Result<(), CompileError> {
        // Vérifier la redéclaration
        if self.globals.borrow().contains(name) {
            return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
        }

        // Réserver le nom global avant de compiler le corps.
        self.globals.borrow_mut().insert(name.to_string());

        // Compiler la fonction dans son propre chunk.
        let function = self.compile_function(name, params, body, line)?;

        // Ajouter l'objet Function dans LE chunk courant.
        let function_constant = self.make_constant(Value::Function(Rc::new(function)))?;

        self.emit_bytes(OpCode::Constant, function_constant, line);

        // Le nom est une constante du chunk courant.
        let name_constant = self.identifier_constant(name)?;

        self.emit_bytes(OpCode::DefineGlobal, name_constant, line);

        Ok(())
    }

    fn compile_function(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Statement],
        line: usize,
    ) -> Result<Function, CompileError> {
        let mut compiler = Compiler::new_function(name.to_string(), Rc::clone(&self.globals));
        for param in params {
            compiler.add_parametre(param)?;
        }

        for statement in body {
            compiler.compile_statement(statement, line)?;
        }

        compiler.emit_opcode(OpCode::Nil, line);
        compiler.emit_opcode(OpCode::Return, line);

        Ok(Function {
            name: name.to_string(),
            arity: compiler.function_arity as usize,
            chunk: compiler.chunk,
        })
    }

    // --------------------------------------------------
    //                  COMPILER_CALL
    // --------------------------------------------------

    fn compile_call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
        line: usize,
    ) -> Result<(), CompileError> {
        // compilé la fonction appllé
        self.compile_expression(callee, line)?;

        //Compile les arguments
        for argument in arguments {
            self.compile_expression(argument, line)?;
        }

        //Vérifier la limite des arguments
        if arguments.len() > u8::MAX as usize {
            return Err(CompileError::TooManyConstants);
        }

        //émettre OP_CALL
        self.emit_bytes(OpCode::Call, arguments.len() as u8, line);

        Ok(())
    }

    // --------------------------------------------------
    //                  COMPILER_RETURN
    // --------------------------------------------------

    fn compile_return(
        &mut self,
        value: Option<&Expression>,
        line: usize,
    ) -> Result<(), CompileError> {
        if !self.in_function {
            return Err(CompileError::ReturnOutsidFunction);
        }
        match value {
            Some(expression) => self.compile_expression(expression, line)?,

            None => Ok(self.emit_opcode(OpCode::Nil, line))?,
        }
        self.emit_opcode(OpCode::Return, line);

        Ok(())
    }
}

//La fonction compilée doit posséder son propre bytecode
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    pub chunk: Chunk,
}
