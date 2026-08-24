use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{BinaryOp, Expression, Literal, Statement, UnaryOp};
use crate::chunk::{Chunk, OpCode};
use crate::error::CompileError;
use crate::value::Value;

#[derive(Clone, Debug)]
pub struct Local {
    pub name: String,
    pub depth: Option<usize>,
    pub slot: u8,
}

#[derive(Debug)]
enum VariableLocation {
    Local(usize),
    Global(usize),
    Upvalue(usize),
}

#[derive(Clone, Debug)]
pub struct LocalTable {
    locals: Vec<Local>,
}

impl LocalTable {
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.locals.len()
    }

    pub fn declare_local(
        &mut self,
        name: &str,
        depth: usize,
    ) -> Result<u8, CompileError> {
        for local in self.locals.iter().rev() {
            if let Some(local_depth) = local.depth {
                if local_depth < depth {
                    break;
                }

                if local.name == name {
                    return Err(
                        CompileError::VariableAlreadyDeclared(name.to_string())
                    );
                }
            }
        }

        if self.locals.len() >= u8::MAX as usize {
            return Err(CompileError::TooManyLocals);
        }

        let slot = self.locals.len() as u8;

        self.locals.push(Local {
            name: name.to_string(),
            depth: None,
            slot,
        });

        Ok(slot)
    }

    pub fn mark_initialized(&mut self, depth: usize) {
        if let Some(local) = self.locals.last_mut() {
            local.depth = Some(depth);
        }
    }

    pub fn resolve_local(
        &self,
        name: &str,
    ) -> Result<Option<u8>, CompileError> {
        for local in self.locals.iter().rev() {
            if local.name != name {
                continue;
            }

            if local.depth.is_none() {
                return Err(
                    CompileError::VariableUseInInitializer(name.to_string())
                );
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

    pub fn cleanup_count(&self, depth: usize) -> usize {
        self.locals
            .iter()
            .filter(|local| {
                matches!(local.depth, Some(d) if d > depth)
            })
            .count()
    }
}

#[derive(Debug, Clone)]
pub struct Upvalue {
    pub index: u8,
    pub is_local: bool,
}

type CompilerContextRef = Rc<RefCell<CompilerContext>>;

#[derive(Debug)]
pub struct CompilerContext {
    pub locals: LocalTable,
    pub upvalues: Vec<Upvalue>,
    pub enclosing: Option<CompilerContextRef>,
}

impl CompilerContext {
    pub fn new() -> Self {
        Self {
            locals: LocalTable::new(),
            upvalues: Vec::new(),
            enclosing: None,
        }
    }

    pub fn new_child(
        enclosing: CompilerContextRef,
    ) -> Self {
        Self {
            locals: LocalTable::new(),
            upvalues: Vec::new(),
            enclosing: Some(enclosing),
        }
    }
}

struct LoopContext {
    continue_target: usize,
    break_jumps: Vec<usize>,
    scope_depth: usize,
}

#[allow(dead_code)]
pub struct Compiler {
    globals: Rc<RefCell<HashMap<String, u8>>>,
    chunk: Chunk,
    context: CompilerContextRef,

    scope_depth: usize,
    loops: Vec<LoopContext>,

    function_name: Option<String>,
    function_arity: u8,
    in_function: bool,
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
        }
    }

    fn new_function(
        name: String,
        globals: Rc<RefCell<HashMap<String, u8>>>,
        enclosing: CompilerContextRef,
    ) -> Self {
        Self {
            globals,
            chunk: Chunk::new(),
            context: Rc::new(RefCell::new(
                CompilerContext::new_child(enclosing)
            )),

            scope_depth: 0,
            loops: Vec::new(),

            function_name: Some(name),
            function_arity: 0,
            in_function: true,
        }
    }

    // ============================================================
    // CONTEXTE
    // ============================================================

    fn locals(&self) -> LocalTable {
        self.context.borrow().locals.clone()
    }

    fn upvalues(&self) -> Vec<Upvalue> {
        self.context.borrow().upvalues.clone()
    }

    // ============================================================
    // CONSTANTES
    // ============================================================

    fn make_constant(
        &mut self,
        value: Value,
    ) -> Result<u8, CompileError> {
        let index = self.chunk.add_constant(value);

        if index > u8::MAX as usize {
            return Err(CompileError::TooManyConstants);
        }

        Ok(index as u8)
    }

    fn identifier_constant(
        &mut self,
        name: &str,
    ) -> Result<u8, CompileError> {
        self.make_constant(Value::String(name.to_string()))
    }

    // ============================================================
    // BYTECODE
    // ============================================================

    fn emit_byte(&mut self, byte: u8, line: usize) {
        self.chunk.write(byte, line);
    }

    fn emit_opcode(&mut self, opcode: OpCode, line: usize) {
        self.emit_byte(opcode.into(), line);
    }

    fn emit_bytes(
        &mut self,
        opcode: OpCode,
        operand: u8,
        line: usize,
    ) {
        self.emit_opcode(opcode, line);
        self.emit_byte(operand, line);
    }

    fn emit_jump(
        &mut self,
        opcode: OpCode,
        line: usize,
    ) -> usize {
        self.emit_opcode(opcode, line);
        self.emit_byte(0xff, line);
        self.emit_byte(0xff, line);

        self.chunk.code.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.code.len() - offset - 2;

        assert!(
            jump <= u16::MAX as usize,
            "Jump trop grand"
        );

        let jump = jump as u16;

        self.chunk.code[offset] =
            (jump >> 8) as u8;

        self.chunk.code[offset + 1] =
            (jump & 0xff) as u8;
    }

    fn emit_loop(
        &mut self,
        loop_start: usize,
        line: usize,
    ) {
        self.emit_opcode(OpCode::Loop, line);

        let offset =
            self.chunk.code.len() + 2 - loop_start;

        assert!(
            offset <= u16::MAX as usize,
            "Loop body too large"
        );

        let offset = offset as u16;

        self.emit_byte(
            (offset >> 8) as u8,
            line,
        );

        self.emit_byte(
            (offset & 0xff) as u8,
            line,
        );
    }

    // ============================================================
    // SCOPE
    // ============================================================

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self, line: usize) {
        self.scope_depth -= 1;

        let count = self
            .context
            .borrow_mut()
            .locals
            .pop_scope(self.scope_depth);

        for _ in 0..count {
            self.emit_opcode(OpCode::Pop, line);
        }
    }

    fn emit_scope_cleanup(
        &mut self,
        target_depth: usize,
        line: usize,
    ) {
        let count = self
            .context
            .borrow()
            .locals
            .cleanup_count(target_depth);

        for _ in 0..count {
            self.emit_opcode(OpCode::Pop, line);
        }
    }

    // ============================================================
    // UPVALUES
    // ============================================================

    fn add_upvalue(
        &mut self,
        index: usize,
        is_local: bool,
    ) -> Result<usize, CompileError> {
        let mut context = self.context.borrow_mut();

        for (i, upvalue) in context.upvalues.iter().enumerate() {
            if upvalue.index as usize == index
                && upvalue.is_local == is_local
            {
                return Ok(i);
            }
        }

        if context.upvalues.len() >= u8::MAX as usize {
            return Err(CompileError::TooManyUpvalues);
        }

        let index_result = context.upvalues.len();

        context.upvalues.push(Upvalue {
            index: index as u8,
            is_local,
        });

        Ok(index_result)
    }

    /*
     * Résolution récursive correcte :
     *
     * enfant
     *   ↓
     * parent local       => is_local = true
     *
     * ou
     *
     * parent upvalue     => is_local = false
     *
     * Exemple :
     *
     * make()
     *   x
     *
     *   get()
     *     get2()
     *       x
     *
     * get capture x
     * get2 capture l'upvalue de get
     */
    fn resolve_upvalue(
        &mut self,
        name: &str,
    ) -> Result<Option<usize>, CompileError> {
        let enclosing = {
            let context = self.context.borrow();

            match &context.enclosing {
                Some(parent) => Rc::clone(parent),
                None => return Ok(None),
            }
        };

        Self::resolve_upvalue_recursive(
            &enclosing,
            name,
        )
        .map(|result| {
            result.map(|(index, is_local)| {
                // Le résultat représente la capture
                // que l'enfant doit faire.
                //
                // is_local = true  => slot du parent
                // is_local = false => upvalue du parent
                //
                // On ajoute cette capture dans self.
                self.add_upvalue(index, is_local)
                    .expect("Too many upvalues")
            })
        })
    }

    fn resolve_upvalue_recursive(
        context: &CompilerContextRef,
        name: &str,
    ) -> Result<Option<(usize, bool)>, CompileError> {
        // --------------------------------------------------------
        // 1. Variable locale du parent immédiat
        // --------------------------------------------------------

        {
            let context_ref = context.borrow();

            if let Some(slot) =
                context_ref.locals.resolve_local(name)?
            {
                return Ok(Some((slot as usize, true)));
            }
        }

        // --------------------------------------------------------
        // 2. Chercher plus loin
        // --------------------------------------------------------

        let enclosing = {
            let context_ref = context.borrow();

            match &context_ref.enclosing {
                Some(parent) => Rc::clone(parent),
                None => return Ok(None),
            }
        };

        let result =
            Self::resolve_upvalue_recursive(
                &enclosing,
                name,
            )?;

        let Some((index, is_local)) = result else {
            return Ok(None);
        };

        // --------------------------------------------------------
        // 3. Le parent doit lui-même capturer la variable
        // --------------------------------------------------------

        let parent_upvalue = {
            let mut context_ref = context.borrow_mut();

            for (i, upvalue) in
                context_ref.upvalues.iter().enumerate()
            {
                if upvalue.index as usize == index
                    && upvalue.is_local == is_local
                {
                    return Ok(Some((i, false)));
                }
            }

            if context_ref.upvalues.len()
                >= u8::MAX as usize
            {
                return Err(
                    CompileError::TooManyUpvalues
                );
            }

            let new_index =
                context_ref.upvalues.len();

            context_ref.upvalues.push(Upvalue {
                index: index as u8,
                is_local,
            });

            new_index
        };

        Ok(Some((parent_upvalue, false)))
    }

    // ============================================================
    // VARIABLES
    // ============================================================

    fn resolve_variable(
        &mut self,
        name: &str,
    ) -> Result<VariableLocation, CompileError> {
        if let Some(slot) = self
            .context
            .borrow()
            .locals
            .resolve_local(name)?
        {
            return Ok(
                VariableLocation::Local(slot as usize)
            );
        }

        if let Some(index) = self.resolve_upvalue(name)? {
            return Ok(
                VariableLocation::Upvalue(index)
            );
        }

        if let Some(index) =
            self.globals.borrow().get(name)
        {
            return Ok(
                VariableLocation::Global(*index as usize)
            );
        }

        Err(
            CompileError::UndefinedVariable(
                name.to_string()
            )
        )
    }

    fn compile_variable_get(
        &mut self,
        name: &str,
        line: usize,
    ) -> Result<(), CompileError> {
        match self.resolve_variable(name)? {
            VariableLocation::Local(slot) => {
                self.emit_bytes(
                    OpCode::GetLocal,
                    slot as u8,
                    line,
                );
            }

            VariableLocation::Global(index) => {
                self.emit_bytes(
                    OpCode::GetGlobal,
                    index as u8,
                    line,
                );
            }

            VariableLocation::Upvalue(index) => {
                self.emit_bytes(
                    OpCode::GetUpvalue,
                    index as u8,
                    line,
                );
            }
        }

        Ok(())
    }

    fn compile_variable_set(
        &mut self,
        name: &str,
        line: usize,
    ) -> Result<(), CompileError> {
        match self.resolve_variable(name)? {
            VariableLocation::Local(slot) => {
                self.emit_bytes(
                    OpCode::SetLocal,
                    slot as u8,
                    line,
                );
            }

            VariableLocation::Global(index) => {
                self.emit_bytes(
                    OpCode::SetGlobal,
                    index as u8,
                    line,
                );
            }

            VariableLocation::Upvalue(index) => {
                self.emit_bytes(
                    OpCode::SetUpvalue,
                    index as u8,
                    line,
                );
            }
        }

        Ok(())
    }

    // ============================================================
    // VARIABLES DECLARATION
    // ============================================================

    fn compile_var(
        &mut self,
        name: &str,
        initializer: Option<&Expression>,
        line: usize,
    ) -> Result<(), CompileError> {
        if self.in_function || self.scope_depth > 0 {
            self.compile_local_var(
                name,
                initializer,
                line,
            )
        } else {
            self.compile_global_var(
                name,
                initializer,
                line,
            )
        }
    }

    fn compile_local_var(
        &mut self,
        name: &str,
        initializer: Option<&Expression>,
        line: usize,
    ) -> Result<(), CompileError> {
        let slot = self
            .context
            .borrow_mut()
            .locals
            .declare_local(
                name,
                self.scope_depth,
            )?;

        match initializer {
            Some(expr) => {
                self.compile_expression(expr, line)?;
            }

            None => {
                self.emit_opcode(
                    OpCode::Nil,
                    line,
                );
            }
        }

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(
                self.scope_depth
            );

        debug_assert_eq!(
            self.context.borrow().locals.len() - 1,
            slot as usize
        );

        Ok(())
    }

    fn compile_global_var(
        &mut self,
        name: &str,
        initializer: Option<&Expression>,
        line: usize,
    ) -> Result<(), CompileError> {
        if self.globals.borrow().contains_key(name) {
            return Err(
                CompileError::VariableAlreadyDeclared(
                    name.to_string()
                )
            );
        }

        let name_constant =
            self.identifier_constant(name)?;

        match initializer {
            Some(expr) => {
                self.compile_expression(
                    expr,
                    line,
                )?;
            }

            None => {
                self.emit_opcode(
                    OpCode::Nil,
                    line,
                );
            }
        }

        self.emit_bytes(
            OpCode::DefineGlobal,
            name_constant,
            line,
        );

        self.globals.borrow_mut().insert(
            name.to_string(),
            name_constant,
        );

        Ok(())
    }

    // ============================================================
    // PARAMÈTRES
    // ============================================================

    fn add_parametre(
        &mut self,
        name: &str,
    ) -> Result<(), CompileError> {
        let slot = self
            .context
            .borrow_mut()
            .locals
            .declare_local(
                name,
                self.scope_depth,
            )?;

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(
                self.scope_depth
            );

        debug_assert_eq!(
            slot,
            self.function_arity
        );

        self.function_arity += 1;

        Ok(())
    }

    // ============================================================
    // CLOSURE
    // ============================================================

    fn emit_closure(
        &mut self,
        function_constant: u8,
        upvalues: &[Upvalue],
        line: usize,
    ) {
        self.emit_bytes(
            OpCode::Closure,
            function_constant,
            line,
        );

        for upvalue in upvalues {
            self.emit_byte(
                if upvalue.is_local {
                    1
                } else {
                    0
                },
                line,
            );

            self.emit_byte(
                upvalue.index,
                line,
            );
        }
    }

    fn compile_function_statement(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Statement],
        line: usize,
    ) -> Result<(), CompileError> {
        // ========================================================
        // FONCTION GLOBALE
        // ========================================================

        if !self.in_function
            && self.scope_depth == 0
        {
            if self.globals.borrow().contains_key(name) {
                return Err(
                    CompileError::VariableAlreadyDeclared(
                        name.to_string()
                    )
                );
            }

            let name_constant =
                self.identifier_constant(name)?;

            // Réserver le nom.
            self.globals.borrow_mut().insert(
                name.to_string(),
                name_constant,
            );

            let function =
                self.compile_function(
                    name,
                    params,
                    body,
                    line,
                )?;

            let function_constant =
                self.make_constant(
                    Value::Function(
                        Rc::new(function.clone())
                    )
                )?;

            self.emit_closure(
                function_constant,
                &function.upvalues,
                line,
            );

            self.emit_bytes(
                OpCode::DefineGlobal,
                name_constant,
                line,
            );

            return Ok(());
        }

        // ========================================================
        // FONCTION LOCALE / NESTED
        // ========================================================

        let function =
            self.compile_function(
                name,
                params,
                body,
                line,
            )?;

        let function_constant =
            self.make_constant(
                Value::Function(
                    Rc::new(function.clone())
                )
            )?;

        self.emit_closure(
            function_constant,
            &function.upvalues,
            line,
        );

        let slot = self
            .context
            .borrow_mut()
            .locals
            .declare_local(
                name,
                self.scope_depth,
            )?;

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(
                self.scope_depth
            );

        debug_assert_eq!(
            self.context.borrow().locals.len() - 1,
            slot as usize
        );

        Ok(())
    }

    fn compile_function(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Statement],
        line: usize,
    ) -> Result<Function, CompileError> {
        let enclosing =
            Rc::clone(&self.context);

        let mut compiler =
            Compiler::new_function(
                name.to_string(),
                Rc::clone(&self.globals),
                enclosing,
            );

        for param in params {
            compiler.add_parametre(param)?;
        }

        for statement in body {
            compiler.compile_statement(
                statement,
                line,
            )?;
        }

        compiler.emit_opcode(
            OpCode::Nil,
            line,
        );

        compiler.emit_opcode(
            OpCode::Return,
            line,
        );

        let upvalues =
            compiler.context.borrow().upvalues.clone();

        Ok(Function {
            name: name.to_string(),
            arity: compiler.function_arity as usize,
            chunk: compiler.chunk,
            upvalue_count: upvalues.len(),
            upvalues,
        })
    }

    // ============================================================
    // EXPRESSION
    // ============================================================

    fn compile_expression(
        &mut self,
        expr: &Expression,
        line: usize,
    ) -> Result<(), CompileError> {
        match expr {
            Expression::Literal(value) => {
                let value = match value {
                    Literal::Number(v) =>
                        Value::Number(*v),

                    Literal::String(v) =>
                        Value::String(v.clone()),

                    Literal::Bool(v) =>
                        Value::Boolean(*v),

                    Literal::Nil =>
                        Value::Nil,
                };

                let constant =
                    self.make_constant(value)?;

                self.emit_bytes(
                    OpCode::Constant,
                    constant,
                    line,
                );
            }

            Expression::Variable(name) => {
                self.compile_variable_get(
                    name,
                    line,
                )?;
            }

            Expression::Binary {
                left,
                operator,
                right,
            } => {
                match operator {
                    BinaryOp::And => {
                        self.compile_logical_and(
                            left,
                            right,
                            line,
                        )?;
                    }

                    BinaryOp::Or => {
                        self.compile_logical_or(
                            left,
                            right,
                            line,
                        )?;
                    }

                    _ => {
                        self.compile_expression(
                            left,
                            line,
                        )?;

                        self.compile_expression(
                            right,
                            line,
                        )?;

                        self.compile_binary(
                            operator.clone(),
                            line,
                        );
                    }
                }
            }

            Expression::Unary {
                operator,
                right,
            } => {
                self.compile_expression(
                    right,
                    line,
                )?;

                match operator {
                    UnaryOp::Negate =>
                        self.emit_opcode(
                            OpCode::Negate,
                            line,
                        ),

                    UnaryOp::Not =>
                        self.emit_opcode(
                            OpCode::Not,
                            line,
                        ),
                }
            }

            Expression::Call {
                callee,
                arguments,
            } => {
                self.compile_call(
                    callee,
                    arguments,
                    line,
                )?;
            }
        }

        Ok(())
    }

    fn compile_call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
        line: usize,
    ) -> Result<(), CompileError> {
        self.compile_expression(
            callee,
            line,
        )?;

        for argument in arguments {
            self.compile_expression(
                argument,
                line,
            )?;
        }

        if arguments.len() > u8::MAX as usize {
            return Err(
                CompileError::TooManyConstants
            );
        }

        self.emit_bytes(
            OpCode::Call,
            arguments.len() as u8,
            line,
        );

        Ok(())
    }

    fn compile_logical_and(
        &mut self,
        left: &Expression,
        right: &Expression,
        line: usize,
    ) -> Result<(), CompileError> {
        self.compile_expression(left, line)?;

        let end_jump =
            self.emit_jump(
                OpCode::JumpIfFalse,
                line,
            );

        self.emit_opcode(
            OpCode::Pop,
            line,
        );

        self.compile_expression(
            right,
            line,
        )?;

        self.patch_jump(end_jump);

        Ok(())
    }

    fn compile_logical_or(
        &mut self,
        left: &Expression,
        right: &Expression,
        line: usize,
    ) -> Result<(), CompileError> {
        self.compile_expression(left, line)?;

        self.emit_opcode(
            OpCode::Not,
            line,
        );

        let end_jump =
            self.emit_jump(
                OpCode::JumpIfFalse,
                line,
            );

        self.emit_opcode(
            OpCode::Not,
            line,
        );

        self.emit_opcode(
            OpCode::Pop,
            line,
        );

        self.compile_expression(
            right,
            line,
        )?;

        self.patch_jump(end_jump);

        Ok(())
    }

    fn compile_binary(
        &mut self,
        operator: BinaryOp,
        line: usize,
    ) {
        let opcode = match operator {
            BinaryOp::Add =>
                OpCode::Add,

            BinaryOp::Subtract =>
                OpCode::Subtract,

            BinaryOp::Multiply =>
                OpCode::Multiply,

            BinaryOp::Divide =>
                OpCode::Divide,

            BinaryOp::Modulo =>
                OpCode::Modulo,

            BinaryOp::Equal =>
                OpCode::Equal,

            BinaryOp::NotEqual => {
                self.emit_opcode(
                    OpCode::Equal,
                    line,
                );

                self.emit_opcode(
                    OpCode::Not,
                    line,
                );

                return;
            }

            BinaryOp::Less =>
                OpCode::Less,

            BinaryOp::LessEqual => {
                self.emit_opcode(
                    OpCode::Greater,
                    line,
                );

                self.emit_opcode(
                    OpCode::Not,
                    line,
                );

                return;
            }

            BinaryOp::Greater =>
                OpCode::Greater,

            BinaryOp::GreaterEqual => {
                self.emit_opcode(
                    OpCode::Less,
                    line,
                );

                self.emit_opcode(
                    OpCode::Not,
                    line,
                );

                return;
            }

            _ => unreachable!(),
        };

        self.emit_opcode(opcode, line);
    }

    // ============================================================
    // RETURN
    // ============================================================

    fn compile_return(
        &mut self,
        value: Option<&Expression>,
        line: usize,
    ) -> Result<(), CompileError> {
        if !self.in_function {
            return Err(
                CompileError::ReturnOutsidFunction
            );
        }

        match value {
            Some(expression) => {
                self.compile_expression(
                    expression,
                    line,
                )?;
            }

            None => {
                self.emit_opcode(
                    OpCode::Nil,
                    line,
                );
            }
        }

        self.emit_opcode(
            OpCode::Return,
            line,
        );

        Ok(())
    }

    // ============================================================
    // IF
    // ============================================================

    fn compile_if(
        &mut self,
        condition: &Expression,
        then_branch: &[Statement],
        else_branch: Option<&Vec<Statement>>,
        line: usize,
    ) -> Result<(), CompileError> {
        self.compile_expression(
            condition,
            line,
        )?;

        let then_jump =
            self.emit_jump(
                OpCode::JumpIfFalse,
                line,
            );

        for statement in then_branch {
            self.compile_statement(
                statement,
                line,
            )?;
        }

        if let Some(else_branch) =
            else_branch
        {
            let else_jump =
                self.emit_jump(
                    OpCode::Jump,
                    line,
                );

            self.patch_jump(then_jump);

            self.emit_opcode(
                OpCode::Pop,
                line,
            );

            for statement in else_branch {
                self.compile_statement(
                    statement,
                    line,
                )?;
            }

            self.patch_jump(else_jump);
        } else {
            self.patch_jump(then_jump);

            self.emit_opcode(
                OpCode::Pop,
                line,
            );
        }

        Ok(())
    }

    // ============================================================
    // WHILE
    // ============================================================

    fn compile_while(
        &mut self,
        condition: &Expression,
        body: &[Statement],
        line: usize,
    ) -> Result<(), CompileError> {
        let loop_start =
            self.chunk.code.len();

        self.compile_expression(
            condition,
            line,
        )?;

        let exit_jump =
            self.emit_jump(
                OpCode::JumpIfFalse,
                line,
            );

        self.emit_opcode(
            OpCode::Pop,
            line,
        );

        self.loops.push(
            LoopContext {
                continue_target: loop_start,
                break_jumps: Vec::new(),
                scope_depth: self.scope_depth,
            }
        );

        self.begin_scope();

        for statement in body {
            self.compile_statement(
                statement,
                line,
            )?;
        }

        self.end_scope(line);

        self.emit_loop(
            loop_start,
            line,
        );

        self.patch_jump(exit_jump);

        self.emit_opcode(
            OpCode::Pop,
            line,
        );

        let loop_context =
            self.loops
                .pop()
                .expect("loop stack underflow");

        for break_jump
            in loop_context.break_jumps
        {
            self.patch_jump(break_jump);
        }

        Ok(())
    }

    // ============================================================
    // BREAK / CONTINUE
    // ============================================================

    fn compile_break(
        &mut self,
        line: usize,
    ) -> Result<(), CompileError> {
        let loop_depth =
            match self.loops.last() {
                Some(loop_context) =>
                    loop_context.scope_depth,

                None =>
                    return Err(
                        CompileError::BreakOutsideLoop
                    ),
            };

        self.emit_scope_cleanup(
            loop_depth,
            line,
        );

        let jump =
            self.emit_jump(
                OpCode::Jump,
                line,
            );

        self.loops
            .last_mut()
            .unwrap()
            .break_jumps
            .push(jump);

        Ok(())
    }

    fn compile_continue(
        &mut self,
        line: usize,
    ) -> Result<(), CompileError> {
        let (continue_target, loop_depth) =
            match self.loops.last() {
                Some(loop_context) => (
                    loop_context.continue_target,
                    loop_context.scope_depth,
                ),

                None =>
                    return Err(
                        CompileError::ContinueOutsideLoop
                    ),
            };

        self.emit_scope_cleanup(
            loop_depth,
            line,
        );

        self.emit_loop(
            continue_target,
            line,
        );

        Ok(())
    }

    // ============================================================
    // STATEMENTS
    // ============================================================

    pub fn compile_statement(
        &mut self,
        stmt: &Statement,
        line: usize,
    ) -> Result<(), CompileError> {
        match stmt {
            Statement::Expression {
                expression,
            } => {
                self.compile_expression(
                    expression,
                    line,
                )?;
            }

            Statement::Let {
                name,
                value,
            } => {
                self.compile_var(
                    name,
                    Some(value),
                    line,
                )?;
            }

            Statement::Block(statements) => {
                self.begin_scope();

                for statement in statements {
                    self.compile_statement(
                        statement,
                        line,
                    )?;
                }

                self.end_scope(line);
            }

            Statement::Assignment {
                name,
                value,
            } => {
                self.compile_expression(
                    value,
                    line,
                )?;

                self.compile_variable_set(
                    name,
                    line,
                )?;
            }

            Statement::Print(expression) => {
                self.compile_expression(
                    expression,
                    line,
                )?;

                self.emit_opcode(
                    OpCode::Print,
                    line,
                );
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_if(
                    condition,
                    then_branch,
                    else_branch.as_ref(),
                    line,
                )?;
            }

            Statement::While {
                condition,
                body,
            } => {
                self.compile_while(
                    condition,
                    body,
                    line,
                )?;
            }

            Statement::Function {
                name,
                params,
                body,
            } => {
                self.compile_function_statement(
                    name,
                    params,
                    body,
                    line,
                )?;
            }

            Statement::Break => {
                self.compile_break(line)?;
            }

            Statement::Continue => {
                self.compile_continue(line)?;
            }

            Statement::Return { value } => {
                self.compile_return(
                    value.as_ref(),
                    line,
                )?;
            }
        }

        Ok(())
    }

    // ============================================================
    // SCRIPT
    // ============================================================

    pub fn compile(
        mut self,
        statements: &[Statement],
        line: usize,
    ) -> Result<Function, CompileError> {
        for statement in statements {
            self.compile_statement(
                statement,
                line,
            )?;
        }

        self.emit_opcode(
            OpCode::Halt,
            line,
        );

        Ok(Function {
            name: "<script>".to_string(),
            arity: 0,
            chunk: self.chunk,
            upvalue_count: 0,
            upvalues: Vec::new(),
        })
    }
}

// ================================================================
// FUNCTION
// ================================================================

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    pub chunk: Chunk,
    pub upvalue_count: usize,
    pub upvalues: Vec<Upvalue>,
}