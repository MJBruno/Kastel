use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::bytecode::chunk::{Chunk, OpCode};
use crate::error::compile_error::CompileError;
use crate::frontend::ast::Statement;
use crate::runtime::function::Function;
use crate::runtime::upvalue::Upvalue;

use super::context::{CompilerContext, CompilerContextRef};
use super::locals::LocalTable;
use super::loops::LoopContext;
use super::variables::Global;

#[allow(dead_code)]
pub struct Compiler {
    pub(crate) globals: Rc<RefCell<HashMap<String, Global>>>,
    pub(crate) chunk: Chunk,
    pub(crate) context: CompilerContextRef,
    pub(crate) scope_depth: usize,
    pub(crate) loops: Vec<LoopContext>,
    pub(crate) function_name: Option<String>,
    pub(crate) function_arity: u8,
    pub(crate) in_function: bool,

    pub(crate) exports: Vec<String>,
    pub(crate) imported_modules: HashSet<String>,

    pub(crate) predeclared_functions: HashSet<String>,

    pub(crate) finally_blocks: Vec<Vec<Statement>>,

    pub(crate) current_line: usize,
    pub(crate) current_column: usize,
}

#[allow(dead_code)]
impl Compiler {
    pub fn new() -> Self {
        Self {
            globals: Rc::new(RefCell::new(HashMap::new())),
            chunk: Chunk::new(),
            context: Rc::new(RefCell::new(CompilerContext::new())),
            scope_depth: 0,
            loops: Vec::new(),
            function_name: None,
            function_arity: 0,
            in_function: false,
            exports: Vec::new(),
            imported_modules: HashSet::new(),
            predeclared_functions: HashSet::new(),
            finally_blocks: Vec::new(),
            current_line: 0,
            current_column: 0,
        }
    }

    pub(crate) fn new_with_globals(
        globals: Rc<RefCell<HashMap<String, Global>>>,
    ) -> Self {
        Self {
            globals,
            chunk: Chunk::new(),
            context: Rc::new(RefCell::new(CompilerContext::new())),
            scope_depth: 0,
            loops: Vec::new(),
            function_name: None,
            function_arity: 0,
            in_function: false,
            exports: Vec::new(),
            imported_modules: HashSet::new(),
            predeclared_functions: HashSet::new(),
            finally_blocks: Vec::new(),
            current_line: 0,
            current_column: 0,
        }
    }

    pub(crate) fn new_function(
        name: String,
        globals: Rc<RefCell<HashMap<String, Global>>>,
        enclosing: CompilerContextRef,
    ) -> Self {
        Self {
            globals,
            chunk: Chunk::new(),
            context: Rc::new(RefCell::new(CompilerContext::new_child(enclosing))),
            scope_depth: 0,
            loops: Vec::new(),
            function_name: Some(name),
            function_arity: 0,
            in_function: true,
            exports: Vec::new(),
            imported_modules: HashSet::new(),
            predeclared_functions: HashSet::new(),
            finally_blocks: Vec::new(),
            current_line: 0,
            current_column: 0,
        }
    }

    // ============================================================
    // MAIN COMPILER
    // ============================================================

    pub fn compile(
        self,
        statements: &[Statement],
    ) -> Result<Function, CompileError> {
        let (function, _) = self.compile_module(statements)?;
        Ok(function)
    }

    pub fn define_native(
        &mut self,
        name: &str,
    ) -> Result<(), CompileError> {
        let constant = self.identifier_constant(name)?;

        self.globals.borrow_mut().insert(
            name.to_string(),
            Global {
                constant,
                mutable: true,
            },
        );

        Ok(())
    }

    // ============================================================
    // GLOBAL FUNCTION PREDECLARATION
    // ============================================================

    fn predeclare_global_functions(
        &mut self,
        statements: &[Statement],
    ) -> Result<(), CompileError> {
        for statement in statements {
            self.predeclare_global_function(statement)?;
        }

        Ok(())
    }

    fn predeclare_global_function(
        &mut self,
        statement: &Statement,
    ) -> Result<(), CompileError> {
        match statement {
            Statement::Positioned { statement, .. } => {
                self.predeclare_global_function(statement)
            }

            Statement::Export { statement } => {
                self.predeclare_global_function(statement)
            }

            Statement::Function { name, .. } => {
                if self.globals.borrow().contains_key(name) {
                    return Err(
                        CompileError::VariableAlreadyDeclared(
                            name.clone(),
                        ),
                    );
                }

                let constant = self.identifier_constant(name)?;

                self.globals.borrow_mut().insert(
                    name.clone(),
                    Global {
                        constant,
                        mutable: true,
                    },
                );

                self.predeclared_functions.insert(name.clone());

                Ok(())
            }

            _ => Ok(()),
        }
    }

    // ============================================================
    // CONTEXTE
    // ============================================================

    pub(crate) fn locals(&self) -> LocalTable {
        self.context.borrow().locals.clone()
    }

    pub(crate) fn upvalues(&self) -> Vec<Upvalue> {
        self.context.borrow().upvalues.clone()
    }

    pub(crate) fn attach_location(
        &self,
        error: CompileError,
    ) -> CompileError {
        match error {
            CompileError::WithLocation { .. } => error,

            other => CompileError::WithLocation {
                line: self.current_line,
                column: self.current_column,
                source: Box::new(other),
            },
        }
    }

    // ============================================================
    // FINALLY
    // ============================================================

    pub(crate) fn push_finally_block(
        &mut self,
        body: &[Statement],
    ) {
        self.finally_blocks.push(body.to_vec());
    }

    pub(crate) fn pop_finally_block(&mut self) {
        self.finally_blocks.pop();
    }

    pub(crate) fn compile_active_finally(
        &mut self,
    ) -> Result<(), CompileError> {
        let finally_blocks = self.finally_blocks.clone();

        for body in finally_blocks.iter().rev() {
            self.begin_scope();

            for statement in body {
                self.compile_statement(statement)?;
            }

            self.end_scope();
        }

        Ok(())
    }

    // ============================================================
    // MODULE
    // ============================================================

    pub fn compile_module(
        mut self,
        statements: &[Statement],
    ) -> Result<(Function, Vec<String>), CompileError> {
        self.predeclare_global_functions(statements)?;

        for statement in statements {
            if let Err(error) = self.compile_statement(statement) {
                return Err(self.attach_location(error));
            }
        }

        self.emit_opcode(OpCode::Halt);

        let local_count = u8::try_from(
            self.context.borrow().locals.max_slots(),
        )
        .map_err(|_| CompileError::TooManyLocals)?;

        let function = Function {
            name: "<script>".to_string(),
            arity: 0,
            chunk: self.chunk,
            local_count: local_count.into(),
            upvalue_count: 0,
            upvalues: Vec::new(),
        };

        Ok((function, self.exports))
    }
}