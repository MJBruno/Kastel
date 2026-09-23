use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::bytecode::chunk::{Chunk, OpCode};
use crate::error::compile_error::CompileError;
use crate::frontend::ast::Statement;
use crate::runtime::function::Function;
// use crate::runtime::upvalue::Upvalue;

use super::context::{CompilerContext, CompilerContextRef};
// use super::locals::LocalTable;
use super::loops::LoopContext;
use super::type_checker::{TypeCheckContext, TypeChecker};
use super::variables::Global;

/// Profondeur maximale d'une expression pour le compilateur et le
/// vérificateur de types (tous deux récursifs). Une chaîne d'opérateurs
/// `1 + 1 + 1 + ...` produit un arbre de la profondeur du nombre de termes :
/// au-delà de cette limite, `CompileError::ExpressionTooDeep` plutôt qu'un
/// débordement de la pile native.
pub const MAX_EXPRESSION_DEPTH: usize = 5_000;

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

    pub(crate) wildcard_imported: bool,

    pub(crate) predeclared_functions: HashSet<String>,

    /// Arités des fonctions globales de même nom (surcharge par arité) :
    /// une fonction peut être déclarée plusieurs fois avec un nombre de
    /// paramètres différent. Toute déclaration de fonction globale émet
    /// `OpCode::Overload` (voir `compile_function_statement`), qui définit,
    /// remplace ou étend selon ce qui existe déjà à l'exécution.
    pub(crate) function_arities: HashMap<String, Vec<usize>>,

    /// Copie du contexte de résolution passé au vérificateur de types
    /// (`None` si le module compile sans résolution d'imports, ou en REPL
    /// sans contexte). Permet à `compile_import` / `compile_from_import` de
    /// savoir si un nom importé est un alias de TYPE pur (aucune valeur à
    /// l'exécution : voir `ModuleTypeInterface::type_aliases`), auquel cas
    /// aucun bytecode n'est émis pour lui.
    pub(crate) type_context: Option<TypeCheckContext>,

    pub(crate) finally_blocks: Vec<Vec<Statement>>,

    pub(crate) current_line: usize,
    pub(crate) current_column: usize,

    /// Profondeur d'expression courante (voir `MAX_EXPRESSION_DEPTH`).
    pub(crate) expression_depth: usize,
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
            function_arities: HashMap::new(),
            type_context: None,
            finally_blocks: Vec::new(),
            current_line: 0,
            current_column: 0,
            expression_depth: 0,
            wildcard_imported: false,
        }
    }

    pub(crate) fn new_with_globals(globals: Rc<RefCell<HashMap<String, Global>>>) -> Self {
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
            function_arities: HashMap::new(),
            type_context: None,
            finally_blocks: Vec::new(),
            current_line: 0,
            current_column: 0,
            expression_depth: 0,
            wildcard_imported: false,
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
            function_arities: HashMap::new(),
            type_context: None,
            finally_blocks: Vec::new(),
            current_line: 0,
            current_column: 0,
            expression_depth: 0,
            wildcard_imported: false,
        }
    }

    // ============================================================
    // MAIN COMPILER
    // ============================================================

    pub fn compile(self, statements: &[Statement]) -> Result<Function, CompileError> {
        let (function, _) = self.compile_module(statements)?;
        Ok(function)
    }

    pub fn compile_with_context(
        self,
        statements: &[Statement],
        context: TypeCheckContext,
    ) -> Result<Function, CompileError> {
        let (function, _) = self.compile_module_with_context(statements, context)?;
        Ok(function)
    }

    pub fn define_native(&mut self, name: &str) -> Result<(), CompileError> {
        let constant = self.identifier_constant(name)?;

        self.globals.borrow_mut().insert(
            name.to_string(),
            Global {
                constant,
                mutable: true,
                native: true,
                is_function: false,
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

    /// Enregistre le nom global d'une fonction, classe ou interface.
    ///
    /// `is_function` distingue une fonction (redéclarable — surcharge par
    /// arité ou redéfinition en REPL) d'une classe ou interface (jamais
    /// redéclarable).
    fn predeclare_global_name(
        &mut self,
        name: &String,
        is_function: bool,
    ) -> Result<(), CompileError> {
        let existing = self.globals.borrow().get(name).cloned();

        if let Some(global) = existing {
            // Une fonction peut toujours redéclarer une fonction existante
            // (surcharge, ou redéfinition en REPL) ; tout le reste (variable,
            // classe, interface) reste protégé.
            if !global.native && !(is_function && global.is_function) {
                return Err(CompileError::VariableAlreadyDeclared(name.clone()));
            }

            self.globals.borrow_mut().insert(
                name.clone(),
                Global {
                    constant: global.constant,
                    mutable: true,
                    native: false,
                    is_function,
                },
            );
        } else {
            let constant = self.identifier_constant(name)?;

            self.globals.borrow_mut().insert(
                name.clone(),
                Global {
                    constant,
                    mutable: true,
                    native: false,
                    is_function,
                },
            );
        }

        self.predeclared_functions.insert(name.clone());

        Ok(())
    }

    fn predeclare_global_function(&mut self, statement: &Statement) -> Result<(), CompileError> {
        match statement {
            Statement::Positioned { statement, .. } => self.predeclare_global_function(statement),

            Statement::Export { statement } => self.predeclare_global_function(statement),

            /*
             * Les fonctions, classes et interfaces globales partagent la
             * même table de symboles de compilation (self.globals) : on
             * les pré-déclare toutes ici, avant de compiler le moindre
             * corps de fonction/méthode. Cela permet :
             *   - les références en avant entre déclarations globales
             *     (ex. une classe qui référence une classe définie plus
             *     bas dans le fichier) ;
             *   - l'auto-référence à l'intérieur d'une méthode (ex. une
             *     méthode `add` de `Point` qui fait `new Point(...)`) —
             *     sans ça, `compile_variable_get(name)` échoue avec
             *     "variable non définie" puisque le nom global n'est
             *     enregistré qu'après compilation du corps.
             */
            // Une fonction globale peut être surchargée par ARITÉ : les
            // déclarations suivantes du même nom s'ajoutent à l'ensemble de
            // surcharges (même arité = erreur).
            Statement::Function { name, params, .. } => {
                if let Some(arities) = self.function_arities.get_mut(name) {
                    if arities.contains(&params.len()) {
                        return Err(CompileError::DuplicateFunction {
                            name: name.clone(),
                            arity: params.len(),
                        });
                    }

                    arities.push(params.len());

                    return Ok(());
                }

                self.function_arities
                    .insert(name.clone(), vec![params.len()]);

                self.predeclare_global_name(name, true)
            }

            Statement::Class { name, .. } | Statement::Interface { name, .. } => {
                self.predeclare_global_name(name, false)
            }

            _ => Ok(()),
        }
    }

    // ============================================================
    // CONTEXTE
    // ============================================================

    // pub(crate) fn locals(&self) -> LocalTable {
    //     self.context.borrow().locals.clone()
    // }

    // pub(crate) fn upvalues(&self) -> Vec<Upvalue> {
    //     self.context.borrow().upvalues.clone()
    // }

    pub(crate) fn attach_location(&self, error: CompileError) -> CompileError {
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

    pub(crate) fn push_finally_block(&mut self, body: &[Statement]) {
        self.finally_blocks.push(body.to_vec());
    }

    pub(crate) fn pop_finally_block(&mut self) {
        self.finally_blocks.pop();
    }

    pub(crate) fn compile_active_finally(&mut self) -> Result<(), CompileError> {
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
        self,
        statements: &[Statement],
    ) -> Result<(Function, Vec<String>), CompileError> {
        self.compile_module_inner(statements, None)
    }

    pub fn compile_module_with_context(
        self,
        statements: &[Statement],
        context: TypeCheckContext,
    ) -> Result<(Function, Vec<String>), CompileError> {
        self.compile_module_inner(statements, Some(context))
    }

    fn compile_module_inner(
        mut self,
        statements: &[Statement],
        context: Option<TypeCheckContext>,
    ) -> Result<(Function, Vec<String>), CompileError> {
        match context.clone() {
            Some(context) => TypeChecker::check_with_context(statements, context)?,
            None => TypeChecker::check(statements)?,
        }

        self.type_context = context;

        self.predeclare_global_functions(statements)?;

        for statement in statements {
            if let Err(error) = self.compile_statement(statement) {
                return Err(self.attach_location(error));
            }
        }

        self.emit_opcode(OpCode::Halt);

        let local_count = u8::try_from(self.context.borrow().locals.max_slots())
            .map_err(|_| CompileError::TooManyLocals)?;

        let function = Function {
            name: "<script>".to_string(),
            arity: 0,
            chunk: Rc::new(self.chunk),
            local_count: local_count.into(),
            upvalue_count: 0,
            upvalues: Vec::new(),
        };

        Ok((function, self.exports))
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
