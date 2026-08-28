use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{BinaryOp, Expression, Literal, Statement, UnaryOp};
use crate::bytecode::{Chunk, OpCode};
use crate::error::CompileError;
use crate::function::Function;
use crate::value::Value;

#[derive(Clone, Debug)]
/// # Local
/// Représente une variable locale connue du compilateur.
/// `name` contient le nom source de la variable, `depth` indique la profondeur
/// de portée à laquelle elle est initialisée et `slot` correspond à sa position
/// dans la pile de la VM."
pub struct Local {
    /// Nom de la variable tel qu'il apparaît dans le programme source.",
    pub name: String,
    /// Profondeur de portée où la variable a été initialisée.
    /// `None` signifie que la variable est encore en cours d'initialisation.",
    pub depth: Option<usize>,
    /// Emplacement de la variable dans la pile des variables locales.",
    pub slot: u8,
    ///Pour distinguer une declaration `const` ou `let`
    pub mutable: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Global {
    constant: u8,
    mutable: bool,
}

#[allow(dead_code)]
/// # VariableLocation
/// Indique l'emplacement où une variable a été résolue par le compilateur.
/// Une variable peut appartenir à la portée `locale`, être globale ou être
/// capturée depuis une portée extérieure sous forme d'upvalue.",
enum VariableLocation {
    Local(usize),
    Global,
    Upvalue(usize),
}

#[derive(Clone, Debug)]
/// Table des variables locales actuellement visibles par le compilateur.",
pub struct LocalTable {
    locals: Vec<Local>,
}
// #[allow(dead_code)]
impl LocalTable {
    /// Crée une table locale vide.",
    pub fn new() -> Self {
        Self { locals: Vec::new() }
    }
    /// Retourne le nombre de variables locales actuellement enregistrées.",
    pub fn len(&self) -> usize {
        self.locals.len()
    }
    /// Déclare une nouvelle variable locale et lui attribue un slot.
    /// La fonction vérifie également les redéclarations dans la portée courante
    /// et refuse de dépasser la capacité représentable par un `u8`.",
    pub fn declare_local(
        &mut self,
        name: &str,
        depth: usize,
        mutable: bool,
    ) -> Result<u8, CompileError> {
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

        if self.locals.len() >= u8::MAX as usize {
            return Err(CompileError::TooManyLocals);
        }

        let slot = self.locals.len() as u8;

        self.locals.push(Local {
            name: name.to_string(),
            depth: None,
            slot,
            mutable,
        });

        Ok(slot)
    }

    pub fn is_mutable(&self, name: &str) -> Result<bool, CompileError> {
        for local in self.locals.iter().rev() {
            if local.name != name {
                continue;
            }

            if local.depth.is_none() {
                return Err(CompileError::VariableUseInInitializer(name.to_string()));
            }

            return Ok(local.mutable);
        }

        Ok(true)
    }

    /// Marque la dernière variable déclarée comme complètement initialisée.",
    pub fn mark_initialized(&mut self, depth: usize) {
        if let Some(local) = self.locals.last_mut() {
            local.depth = Some(depth);
        }
    }

    /// Recherche une variable locale depuis la portée la plus proche.
    /// Retourne son slot si elle existe, ou `None` lorsqu'elle n'est pas locale.",
    pub fn resolve_local(&self, name: &str) -> Result<Option<u8>, CompileError> {
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

    /// Supprime les variables appartenant aux portées qui viennent de se terminer.
    /// Retourne le nombre de variables retirées afin que le compilateur puisse
    /// générer autant d'instructions `Pop` dans le bytecode.",
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

    /// Compte les variables qui doivent être retirées avant un saut hors de portée.",
    pub fn cleanup_count(&self, depth: usize) -> usize {
        self.locals
            .iter()
            .filter(|local| matches!(local.depth, Some(d) if d > depth))
            .count()
    }
}

#[derive(Debug, Clone)]
/// Décrit une variable capturée par une closure.
/// `index` désigne soit un slot local, soit un index d'upvalue du contexte parent.
/// `is_local` indique lequel des deux cas s'applique.",
pub struct Upvalue {
    /// Index du slot local ou de l'upvalue dans le contexte source.",
    pub index: u8,
    /// Vrai lorsque l'upvalue capture directement une variable locale du parent.",
    pub is_local: bool,
}

type CompilerContextRef = Rc<RefCell<CompilerContext>>;
#[allow(dead_code)]
#[derive(Debug)]
/// Contexte lexical utilisé pendant la compilation d'une fonction ou d'un script.
/// Il contient les variables locales, les upvalues et un lien vers le contexte
/// de compilation de la fonction englobante.",
pub struct CompilerContext {
    /// Variables locales appartenant à ce contexte de compilation.",
    pub locals: LocalTable,
    /// Variables capturées depuis les contextes englobants.",
    pub upvalues: Vec<Upvalue>,
    /// Contexte de compilation de la fonction parente, s'il existe.",
    pub enclosing: Option<CompilerContextRef>,
}

impl CompilerContext {
    /// Crée un contexte racine sans fonction englobante.",
    pub fn new() -> Self {
        Self {
            locals: LocalTable::new(),
            upvalues: Vec::new(),
            enclosing: None,
        }
    }

    /// Crée un contexte enfant relié au contexte de compilation parent.",
    pub fn new_child(enclosing: CompilerContextRef) -> Self {
        Self {
            locals: LocalTable::new(),
            upvalues: Vec::new(),
            enclosing: Some(enclosing),
        }
    }
}
/// État de compilation d'une boucle actuellement active.
/// Cet état permet de résoudre correctement `break` et `continue` après
/// génération du bytecode.",
struct LoopContext {
    /// Offset de bytecode vers lequel `continue` doit revenir.",
    continue_target: usize,
    /// Liste des sauts `break` qui devront être corrigés à la fin de la boucle.",
    break_jumps: Vec<usize>,
    /// Profondeur de portée à laquelle la boucle a été créée.",
    scope_depth: usize,
}

#[allow(dead_code)]
/// Compile l'AST du langage en bytecode exécutable par la machine virtuelle.
/// Le compilateur gère notamment les variables, les fonctions, les closures,
/// les portées lexicales, les conditions, les boucles et les expressions.",
pub struct Compiler {
    /// Table partagée des variables globales et de leurs constantes de nom.",
    globals: Rc<RefCell<HashMap<String, Global>>>,
    /// Chunk contenant le bytecode et les constantes produits par ce compilateur.",
    chunk: Chunk,
    /// Contexte lexical courant utilisé pour résoudre les variables et captures."
    context: CompilerContextRef,
    /// Profondeur de portée lexicale actuellement compilée.",
    scope_depth: usize,
    /// Pile des boucles imbriquées actuellement en cours de compilation.",
    loops: Vec<LoopContext>,
    /// Nom de la fonction actuellement compilée, lorsqu'il y en a une.",
    function_name: Option<String>,
    /// Nombre de paramètres de la fonction courante.",
    function_arity: u8,
    /// Indique si le compilateur se trouve à l'intérieur d'une fonction.",
    in_function: bool,
}

#[allow(dead_code)]
/// Crée un compilateur racine prêt à compiler un script.",
impl Compiler {
    /// Crée un compilateur indépendant pour une nouvelle fonction.
    /// Le nouveau compilateur partage les globales avec son parent et conserve
    /// une référence vers le contexte englobant afin de résoudre les captures.",
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
        }
    }

    // ============================================================
    //                      MAIN_COMPILER
    // ============================================================

    pub fn compile(mut self, statements: &[Statement]) -> Result<Function, CompileError> {
        for statement in statements {
            self.compile_statement(statement)?;
        }

        self.emit_opcode(OpCode::Halt);

        Ok(Function {
            name: "<script>".to_string(),
            arity: 0,
            chunk: self.chunk,
            upvalue_count: 0,
            upvalues: Vec::new(),
        })
    }

    /// Enregistre une fonction native dans la table des symboles globaux.",
    pub fn define_native(&mut self, name: &str) -> Result<(), CompileError> {
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
    //                      CONTEXTE
    // ============================================================
    /// Retourne une copie de la table des variables locales courantes.",
    fn locals(&self) -> LocalTable {
        self.context.borrow().locals.clone()
    }
    /// Retourne une copie des upvalues du contexte courant.",
    fn upvalues(&self) -> Vec<Upvalue> {
        self.context.borrow().upvalues.clone()
    }

    // ============================================================
    //                      CONSTANTES
    // ============================================================
    /// Ajoute une valeur à la table des constantes et retourne son index sur 8 bits.",
    fn make_constant(&mut self, value: Value) -> Result<u8, CompileError> {
        let index = self.chunk.add_constant(value);

        if index > u8::MAX as usize {
            return Err(CompileError::TooManyConstants);
        }

        Ok(index as u8)
    }

    fn identifier_constant(&mut self, name: &str) -> Result<u8, CompileError> {
        self.make_constant(Value::String(name.to_string()))
    }

    // ============================================================
    // BYTECODE
    // ============================================================

    fn emit_byte(&mut self, byte: u8) {
        self.chunk.write(byte);
    }

    fn emit_opcode(&mut self, opcode: OpCode) {
        self.emit_byte(opcode.into());
    }

    fn emit_bytes(&mut self, opcode: OpCode, operand: u8) {
        self.emit_opcode(opcode);
        self.emit_byte(operand);
    }

    fn emit_jump(&mut self, opcode: OpCode) -> usize {
        self.emit_opcode(opcode);
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        self.chunk.code.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.code.len() - offset - 2;
        assert!(jump <= u16::MAX as usize, "Jump trop grand");
        let jump = jump as u16;
        self.chunk.code[offset] = (jump >> 8) as u8;
        self.chunk.code[offset + 1] = (jump & 0xff) as u8;
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_opcode(OpCode::Loop);
        let offset = self.chunk.code.len() + 2 - loop_start;
        assert!(offset <= u16::MAX as usize, "Loop body too large");
        let offset = offset as u16;
        self.emit_byte((offset >> 8) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }

    // ============================================================
    // SCOPE
    // ============================================================

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        let count = self.context.borrow_mut().locals.pop_scope(self.scope_depth);

        for _ in 0..count {
            self.emit_opcode(OpCode::Pop);
        }
    }

    fn emit_scope_cleanup(&mut self, target_depth: usize) {
        let count = self.context.borrow().locals.cleanup_count(target_depth);

        for _ in 0..count {
            self.emit_opcode(OpCode::Pop);
        }
    }

    // ============================================================
    // UPVALUES
    // ============================================================

    fn add_upvalue(&mut self, index: usize, is_local: bool) -> Result<usize, CompileError> {
        let mut context = self.context.borrow_mut();

        for (i, upvalue) in context.upvalues.iter().enumerate() {
            if upvalue.index as usize == index && upvalue.is_local == is_local {
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
    fn resolve_upvalue(&mut self, name: &str) -> Result<Option<usize>, CompileError> {
        let enclosing = {
            let context = self.context.borrow();

            match &context.enclosing {
                Some(parent) => Rc::clone(parent),
                None => return Ok(None),
            }
        };

        Self::resolve_upvalue_recursive(&enclosing, name).map(|result| {
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

            if let Some(slot) = context_ref.locals.resolve_local(name)? {
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

        let result = Self::resolve_upvalue_recursive(&enclosing, name)?;

        let Some((index, is_local)) = result else {
            return Ok(None);
        };

        // --------------------------------------------------------
        // 3. Le parent doit lui-même capturer la variable
        // --------------------------------------------------------

        let parent_upvalue = {
            let mut context_ref = context.borrow_mut();

            for (i, upvalue) in context_ref.upvalues.iter().enumerate() {
                if upvalue.index as usize == index && upvalue.is_local == is_local {
                    return Ok(Some((i, false)));
                }
            }

            if context_ref.upvalues.len() >= u8::MAX as usize {
                return Err(CompileError::TooManyUpvalues);
            }

            let new_index = context_ref.upvalues.len();

            context_ref.upvalues.push(Upvalue {
                index: index as u8,
                is_local,
            });

            new_index
        };

        Ok(Some((parent_upvalue, false)))
    }

    // ============================================================
    //                      VARIABLES
    // ============================================================

    fn resolve_variable(&self, name: &str) -> Result<VariableLocation, CompileError> {
        if let Some(slot) = self.context.borrow().locals.resolve_local(name)? {
            return Ok(VariableLocation::Local(slot as usize));
        }

        if self.globals.borrow().contains_key(name) {
            return Ok(VariableLocation::Global);
        }

        Err(CompileError::UndefinedVariable(name.to_string()))
    }

    fn compile_variable_get(&mut self, name: &str) -> Result<(), CompileError> {
        match self.resolve_variable(name)? {
            VariableLocation::Local(slot) => {
                self.emit_bytes(OpCode::GetLocal, slot as u8);
            }

            VariableLocation::Global => {
                // IMPORTANT :
                // la constante doit appartenir au chunk courant
                let name_constant = self.identifier_constant(name)?;

                self.emit_bytes(OpCode::GetGlobal, name_constant);
            }
            VariableLocation::Upvalue(slot) => {
                self.emit_bytes(OpCode::GetUpvalue, slot as u8);
            }
        }

        Ok(())
    }

    fn compile_variable_set(&mut self, name: &str) -> Result<(), CompileError> {
        match self.resolve_variable(name)? {
            VariableLocation::Local(slot) => {
                let mutable = self.context.borrow().locals.is_mutable(name)?;

                if let mutable = mutable
                    && !mutable
                {
                    return Err(CompileError::AssignmentToConstant(name.to_string()));
                }

                self.emit_bytes(OpCode::SetLocal, slot as u8);
            }

            VariableLocation::Global => {
                let is_mutable = {
                    let globals = self.globals.borrow();

                    globals.get(name).map(|global| global.mutable)
                };

                if let Some(false) = is_mutable {
                    return Err(CompileError::AssignmentToConstant(name.to_string()));
                }

                let name_constant = self.identifier_constant(name)?;

                self.emit_bytes(OpCode::SetGlobal, name_constant);
            }

            VariableLocation::Upvalue(slot) => {
                /*
                 * Une variable capturée conserve sa mutabilité
                 * depuis le Local d'origine.
                 *
                 * La vérification doit donc être faite pendant
                 * la résolution de l'upvalue.
                 */
                self.emit_bytes(OpCode::SetUpvalue, slot as u8);
            }
        }

        Ok(())
    }

    // ============================================================
    //                      VARIABLES DECLARATION
    // ============================================================

    fn compile_var(
        &mut self,
        name: &str,
        initializer: Option<&Expression>,
        mutable: bool,
    ) -> Result<(), CompileError> {
        if self.in_function || self.scope_depth > 0 {
            self.compile_local_var(name, initializer, mutable)
        } else {
            self.compile_global_var(name, initializer, mutable)
        }
    }
    fn compile_local_var(
        &mut self,
        name: &str,
        initializer: Option<&Expression>,
        mutable: bool,
    ) -> Result<(), CompileError> {
        let slot =
            self.context
                .borrow_mut()
                .locals
                .declare_local(name, self.scope_depth, mutable)?;

        match initializer {
            Some(expr) => {
                self.compile_expression(expr)?;
            }

            None => {
                self.emit_opcode(OpCode::Nil);
            }
        }

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(self.scope_depth);

        debug_assert_eq!(self.context.borrow().locals.len() - 1, slot as usize);

        Ok(())
    }

    fn compile_global_var(
        &mut self,
        name: &str,
        initializer: Option<&Expression>,
        mutable: bool,
    ) -> Result<(), CompileError> {
        if self.globals.borrow().contains_key(name) {
            return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
        }

        let name_constant = self.identifier_constant(name)?;

        match initializer {
            Some(expr) => {
                self.compile_expression(expr)?;
            }

            None => {
                self.emit_opcode(OpCode::Nil);
            }
        }

        self.emit_bytes(OpCode::DefineGlobal, name_constant);

        self.globals.borrow_mut().insert(
            name.to_string(),
            Global {
                constant: name_constant,
                mutable,
            },
        );

        Ok(())
    }

    // ============================================================
    //                      PARAMÈTRES
    // ============================================================

    fn add_parametre(&mut self, name: &str) -> Result<(), CompileError> {
        let slot = self
            .context
            .borrow_mut()
            .locals
            .declare_local(name, self.scope_depth, true)?;

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(self.scope_depth);

        debug_assert_eq!(slot, self.function_arity);

        self.function_arity += 1;

        Ok(())
    }

    // ============================================================
    //                      CLOSURE
    // ============================================================

    fn emit_closure(&mut self, function_constant: u8, upvalues: &[Upvalue]) {
        self.emit_bytes(OpCode::Closure, function_constant);

        for upvalue in upvalues {
            self.emit_byte(if upvalue.is_local { 1 } else { 0 });

            self.emit_byte(upvalue.index);
        }
    }

    fn compile_function_statement(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Statement],
    ) -> Result<(), CompileError> {
        // ========================================================
        // FONCTION GLOBALE
        // ========================================================

        if !self.in_function && self.scope_depth == 0 {
            if self.globals.borrow().contains_key(name) {
                return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
            }

            let name_constant = self.identifier_constant(name)?;

            // Réserver le nom.
            self.globals.borrow_mut().insert(
                name.to_string(),
                Global {
                    constant: name_constant,
                    mutable: true,
                },
            );

            let function = self.compile_function(name, params, body)?;

            let function_constant =
                self.make_constant(Value::Function(Rc::new(function.clone())))?;

            self.emit_closure(function_constant, &function.upvalues);

            self.emit_bytes(OpCode::DefineGlobal, name_constant);

            return Ok(());
        }

        // ========================================================
        // FONCTION LOCALE / NESTED
        // ========================================================

        let function = self.compile_function(name, params, body)?;

        let function_constant = self.make_constant(Value::Function(Rc::new(function.clone())))?;

        self.emit_closure(function_constant, &function.upvalues);

        let slot = self
            .context
            .borrow_mut()
            .locals
            .declare_local(name, self.scope_depth, true)?;

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(self.scope_depth);

        debug_assert_eq!(self.context.borrow().locals.len() - 1, slot as usize);

        Ok(())
    }

    // ========================================================
    //                      COMPILE_FONCTION
    // ========================================================

    fn compile_function(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Statement],
    ) -> Result<Function, CompileError> {
        let enclosing = Rc::clone(&self.context);

        let mut compiler =
            Compiler::new_function(name.to_string(), Rc::clone(&self.globals), enclosing);

        for param in params {
            compiler.add_parametre(param)?;
        }

        for statement in body {
            compiler.compile_statement(statement)?;
        }

        compiler.emit_opcode(OpCode::Nil);

        compiler.emit_opcode(OpCode::Return);

        let upvalues = compiler.context.borrow().upvalues.clone();

        Ok(Function {
            name: name.to_string(),
            arity: compiler.function_arity as usize,
            chunk: compiler.chunk,
            upvalue_count: upvalues.len(),
            upvalues,
        })
    }

    // ============================================================
    //                      EXPRESSION
    // ============================================================
    #[allow(unused_variables)]
    fn compile_expression(&mut self, expr: &Expression) -> Result<(), CompileError> {
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
                self.compile_call(callee, arguments)?;
            }
            Expression::Member { object, property } => todo!(),
            Expression::Index { object, index } => todo!(),
            Expression::Array(expressions) => todo!(),
            Expression::Object(items) => todo!(),
        }

        Ok(())
    }

    fn compile_call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
    ) -> Result<(), CompileError> {
        self.compile_expression(callee)?;

        for argument in arguments {
            self.compile_expression(argument)?;
        }

        if arguments.len() > u8::MAX as usize {
            return Err(CompileError::TooManyConstants);
        }

        self.emit_bytes(OpCode::Call, arguments.len() as u8);

        Ok(())
    }

    fn compile_logical_and(
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

    fn compile_logical_or(
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

    fn compile_binary(&mut self, operator: BinaryOp) {
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

    // ============================================================
    //                      RETURN
    // ============================================================

    fn compile_return(&mut self, value: Option<&Expression>) -> Result<(), CompileError> {
        if !self.in_function {
            return Err(CompileError::ReturnOutsidFunction);
        }

        match value {
            Some(expression) => {
                self.compile_expression(expression)?;
            }

            None => {
                self.emit_opcode(OpCode::Nil);
            }
        }

        self.emit_opcode(OpCode::Return);

        Ok(())
    }

    // ============================================================
    //                      COMPILE_IF
    // ============================================================

    fn compile_if(
        &mut self,
        condition: &Expression,
        then_branch: &[Statement],
        else_branch: Option<&Vec<Statement>>,
    ) -> Result<(), CompileError> {
        self.compile_expression(condition)?;

        let then_jump = self.emit_jump(OpCode::JumpIfFalse);

        for statement in then_branch {
            self.compile_statement(statement)?;
        }

        if let Some(else_branch) = else_branch {
            let else_jump = self.emit_jump(OpCode::Jump);

            self.patch_jump(then_jump);

            self.emit_opcode(OpCode::Pop);

            for statement in else_branch {
                self.compile_statement(statement)?;
            }

            self.patch_jump(else_jump);
        } else {
            self.patch_jump(then_jump);

            self.emit_opcode(OpCode::Pop);
        }

        Ok(())
    }

    // ============================================================
    //                      WHILE
    // ============================================================

    fn compile_while(
        &mut self,
        condition: &Expression,
        body: &[Statement],
    ) -> Result<(), CompileError> {
        let loop_start = self.chunk.code.len();

        self.compile_expression(condition)?;

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_opcode(OpCode::Pop);

        self.loops.push(LoopContext {
            continue_target: loop_start,
            break_jumps: Vec::new(),
            scope_depth: self.scope_depth,
        });

        self.begin_scope();

        for statement in body {
            self.compile_statement(statement)?;
        }

        self.end_scope();

        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);

        self.emit_opcode(OpCode::Pop);

        let loop_context = self.loops.pop().expect("loop stack underflow");

        for break_jump in loop_context.break_jumps {
            self.patch_jump(break_jump);
        }

        Ok(())
    }

    // ============================================================
    //                      BREAK / CONTINUE
    // ============================================================

    fn compile_break(&mut self) -> Result<(), CompileError> {
        let loop_depth = match self.loops.last() {
            Some(loop_context) => loop_context.scope_depth,

            None => return Err(CompileError::BreakOutsideLoop),
        };

        self.emit_scope_cleanup(loop_depth);

        let jump = self.emit_jump(OpCode::Jump);

        self.loops.last_mut().unwrap().break_jumps.push(jump);

        Ok(())
    }

    fn compile_continue(&mut self) -> Result<(), CompileError> {
        let (continue_target, loop_depth) = match self.loops.last() {
            Some(loop_context) => (loop_context.continue_target, loop_context.scope_depth),

            None => return Err(CompileError::ContinueOutsideLoop),
        };

        self.emit_scope_cleanup(loop_depth);

        self.emit_loop(continue_target);

        Ok(())
    }

    // ============================================================
    //                      STATEMENTS
    // ============================================================

    pub fn compile_statement(&mut self, stmt: &Statement) -> Result<(), CompileError> {
        match stmt {
            Statement::Expression { expression } => {
                self.compile_expression(expression)?;
            }

            Statement::Let {
                name,
                value,
                mutable,
            } => {
                self.compile_var(name, Some(value), *mutable)?;
            }

            Statement::Block(statements) => {
                self.begin_scope();

                for statement in statements {
                    self.compile_statement(statement)?;
                }

                self.end_scope();
            }

            Statement::Assignment { name, value } => {
                self.compile_expression(value)?;

                self.compile_variable_set(name)?;
            }

            Statement::Print(expression) => {
                self.compile_expression(expression)?;

                self.emit_opcode(OpCode::Print);
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_if(condition, then_branch, else_branch.as_ref())?;
            }

            Statement::While { condition, body } => {
                self.compile_while(condition, body)?;
            }

            Statement::Function { name, params, body } => {
                self.compile_function_statement(name, params, body)?;
            }

            Statement::Break => {
                self.compile_break()?;
            }

            Statement::Continue => {
                self.compile_continue()?;
            }

            Statement::Return { value } => {
                self.compile_return(value.as_ref())?;
            }
        }

        Ok(())
    }
}
