use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::bytecode::chunk::{Chunk, OpCode};
use crate::error::compile_error::CompileError;
use crate::frontend::ast::Statement;
use crate::runtime::function::Function;
// use crate::runtime::upvalue::Upvalue;

use super::call_resolution::{CallSite, ResolvedCall, ResolvedCallTable};
use super::context::{CompilerContext, CompilerContextRef};
// use super::locals::LocalTable;
use super::loops::LoopContext;
use super::overloads::OverloadSet;
use super::type_checker::{TypeCheckContext, TypeChecker};
use super::variables::Global;

/// Profondeur maximale d'une expression pour le compilateur et le
/// vérificateur de types (tous deux récursifs). Une chaîne d'opérateurs
/// `1 + 1 + 1 + ...` produit un arbre de la profondeur du nombre de termes :
/// au-delà de cette limite, `CompileError::ExpressionTooDeep` plutôt qu'un
/// débordement de la pile native.
pub const MAX_EXPRESSION_DEPTH: usize = 1_024;

/// Un `try` ouvert autour du code compilé. Sert à `return`, `?`, `break` et
/// `continue`, qui doivent dépiler les handlers et exécuter les `finally` des
/// `try` qu'ils quittent.
#[derive(Clone)]
pub(crate) struct TryContext {
    /// Corps du `finally` (`None` : `try/catch` sans `finally`).
    pub(crate) finally: Option<Vec<Statement>>,

    /// Le handler d'exceptions est-il encore empilé à cet endroit du code ?
    /// Vrai dans le corps du `try`, et dans le `catch` seulement s'il existe
    /// un `finally` (sinon la VM retire le handler à l'entrée du `catch`).
    pub(crate) handler_active: bool,

    /// Nombre de boucles ouvertes à l'entrée du `try` : distingue les `try`
    /// qu'un `break`/`continue` quitte de ceux qui entourent la boucle.
    pub(crate) loops_len: usize,
}

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
    pub(crate) function_arities: HashMap<String, OverloadSet<usize>>,

    /// Copie du contexte de résolution passé au vérificateur de types
    /// (`None` si le module compile sans résolution d'imports, ou en REPL
    /// sans contexte). Permet à `compile_import` / `compile_from_import` de
    /// savoir si un nom importé est un alias de TYPE pur (aucune valeur à
    /// l'exécution : voir `ModuleTypeInterface::type_aliases`), auquel cas
    /// aucun bytecode n'est émis pour lui.
    pub(crate) type_context: Option<TypeCheckContext>,

    /// `try` actuellement ouverts autour du code en cours de compilation
    /// (du plus externe au plus interne), dans la fonction courante.
    pub(crate) try_contexts: Vec<TryContext>,

    pub(crate) current_line: usize,
    pub(crate) current_column: usize,

    /// Profondeur d'expression courante (voir `MAX_EXPRESSION_DEPTH`).
    pub(crate) expression_depth: usize,

    /// Décisions sémantiques des call-sites produites par le TypeChecker.
    /// Partagées avec les sous-compilateurs de fonctions afin que la
    /// génération de bytecode ne redéduise pas les intrinsèques par leur nom.
    pub(crate) resolved_calls: Rc<ResolvedCallTable>,
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
            try_contexts: Vec::new(),
            current_line: 0,
            current_column: 0,
            expression_depth: 0,
            resolved_calls: Rc::new(ResolvedCallTable::default()),
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
            try_contexts: Vec::new(),
            current_line: 0,
            current_column: 0,
            expression_depth: 0,
            resolved_calls: Rc::new(ResolvedCallTable::default()),
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
            try_contexts: Vec::new(),
            current_line: 0,
            current_column: 0,
            expression_depth: 0,
            resolved_calls: Rc::new(ResolvedCallTable::default()),
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

    pub(crate) fn resolved_call(
        &self,
        line: usize,
        column: usize,
    ) -> Option<&ResolvedCall> {
        self.resolved_calls.get(CallSite::new(line, column))
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
                    if arities.insert_unique(params.len()).is_err() {
                        return Err(CompileError::DuplicateFunction {
                            name: name.clone(),
                            arity: params.len(),
                        });
                    }

                    return Ok(());
                }

                self.function_arities
                    .insert(name.clone(), OverloadSet::from_one(params.len()));

                self.predeclare_global_name(name, true)
            }

            Statement::Class { name, .. }
            | Statement::Interface { name, .. }
            | Statement::Enum { name, .. } => self.predeclare_global_name(name, false),

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

    pub(crate) fn push_try_context(&mut self, finally: Option<&[Statement]>) {
        self.try_contexts.push(TryContext {
            finally: finally.map(<[Statement]>::to_vec),
            handler_active: true,
            loops_len: self.loops.len(),
        });
    }

    pub(crate) fn pop_try_context(&mut self) {
        self.try_contexts.pop();
    }

    pub(crate) fn set_try_handler_active(&mut self, active: bool) {
        if let Some(context) = self.try_contexts.last_mut() {
            context.handler_active = active;
        }
    }

    /// Quitte les `try` d'indice >= `down_to` (du plus interne au plus
    /// externe) : dépile leur handler s'il est encore actif, puis inline leur
    /// `finally`. Le corps d'un `finally` est compilé HORS de son propre
    /// `try` : un `return`/`break` qu'il contient ne le ré-inline pas
    /// (récursion infinie du compilateur) et ne dépile pas un handler déjà
    /// dépilé (double exécution du `finally`).
    pub(crate) fn unwind_try_contexts(&mut self, down_to: usize) -> Result<(), CompileError> {
        let mut index = self.try_contexts.len();

        while index > down_to {
            index -= 1;

            let context = self.try_contexts[index].clone();

            if context.handler_active {
                self.emit_opcode(OpCode::PopExceptionHandler);
            }

            if let Some(body) = &context.finally {
                let saved = self.try_contexts.split_off(index);

                self.begin_scope();

                let mut result = Ok(());
                for statement in body {
                    result = self.compile_statement(statement);
                    if result.is_err() {
                        break;
                    }
                }

                self.end_scope();
                self.try_contexts.extend(saved);

                result?;
            }
        }

        Ok(())
    }

    /// `break` / `continue` : quitte uniquement les `try` ouverts DANS la
    /// boucle courante.
    pub(crate) fn unwind_try_contexts_in_loop(&mut self) -> Result<(), CompileError> {
        let loops_len = self.loops.len();

        let first = self
            .try_contexts
            .iter()
            .position(|context| context.loops_len >= loops_len)
            .unwrap_or(self.try_contexts.len());

        self.unwind_try_contexts(first)
    }

    /// Émet un `return` dont la valeur est déjà sur la pile, en exécutant
    /// d'abord les `finally` actifs. La valeur devient une locale cachée : les
    /// `finally` inlinés déclarent leurs propres locales juste au-dessus, avec
    /// des numéros de slot corrects (avant, la valeur temporaire décalait les
    /// slots et un `finally` avec `let` lisait la valeur de retour).
    ///
    /// `in_expression` : `return` déclenché au milieu d'une expression (opérateur
    /// `?`), donc avec d'éventuelles valeurs temporaires sous la valeur de
    /// retour. Le numéro de slot d'une locale cachée serait alors faux : la
    /// valeur est simplement laissée au sommet de la pile pendant les
    /// `finally` (limite connue : un `finally` inliné qui déclare une locale
    /// dans ce cas précis garde l'ancien décalage de slots).
    pub(crate) fn emit_return_through_finally(
        &mut self,
        in_expression: bool,
    ) -> Result<(), CompileError> {
        if self.try_contexts.is_empty() {
            self.emit_opcode(OpCode::Return);
            return Ok(());
        }

        if in_expression {
            self.unwind_try_contexts(0)?;
            self.emit_opcode(OpCode::Return);
            return Ok(());
        }

        self.begin_scope();

        let slot = self.declare_existing_local("__return_value", true)?;

        self.unwind_try_contexts(0)?;

        self.emit_bytes(OpCode::GetLocal, slot);
        self.emit_opcode(OpCode::Return);

        // Le `Return` termine le frame : aucun `Pop` à émettre.
        self.discard_scope();

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
        let type_check =
            TypeChecker::check_for_compiler(statements, context.clone())?;
        self.resolved_calls = Rc::new(type_check.resolved_calls);
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
            is_async: false,
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
