use std::cell::RefCell;
use std::collections::HashMap;
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
/// Compile l'AST du langage en bytecode exécutable par la machine virtuelle.
/// Le compilateur gère notamment les variables, les fonctions, les closures,
/// les portées lexicales, les conditions, les boucles et les expressions.
///
/// L'implémentation est répartie sur plusieurs fichiers (voir `compile/mod.rs`) :
/// chacun ajoute un bloc `impl Compiler` dédié à une responsabilité précise
/// (variables, upvalues, scopes, émission de bytecode, déclarations, fonctions,
/// expressions, contrôle de flux, boucles, statements). Ce fichier ne contient
/// que la définition de la struct et son API d'entrée/sortie.
pub struct Compiler {
    /// Table partagée des variables globales et de leurs constantes de nom.
    pub(crate) globals: Rc<RefCell<HashMap<String, Global>>>,
    /// Chunk contenant le bytecode et les constantes produits par ce compilateur.
    pub(crate) chunk: Chunk,
    /// Contexte lexical courant utilisé pour résoudre les variables et captures.
    pub(crate) context: CompilerContextRef,
    /// Profondeur de portée lexicale actuellement compilée.
    pub(crate) scope_depth: usize,
    /// Pile des boucles imbriquées actuellement en cours de compilation.
    pub(crate) loops: Vec<LoopContext>,
    /// Nom de la fonction actuellement compilée, lorsqu'il y en a une.
    pub(crate) function_name: Option<String>,
    /// Nombre de paramètres de la fonction courante.
    pub(crate) function_arity: u8,
    /// Indique si le compilateur se trouve à l'intérieur d'une fonction.
    pub(crate) in_function: bool,

    pub(crate) exports: Vec<String>,

    /// Position source (ligne, colonne) du statement en cours de
    /// compilation. Mise à jour uniquement au passage d'un
    /// `Statement::Positioned` (voir statements.rs) — c'est ce que lit
    /// `emit_byte` pour alimenter `chunk.lines`/`chunk.columns`, et ce que
    /// `compile()`/`compile_module()`/`compile_function()` utilisent pour
    /// enrichir un `CompileError` avec sa position au moment où il
    /// s'échappe vers l'appelant.
    pub(crate) current_line: usize,
    pub(crate) current_column: usize,
}

#[allow(dead_code)]
impl Compiler {
    /// Crée un compilateur racine prêt à compiler un script.
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

            current_line: 0,
            current_column: 0,
        }
    }

    /// Crée un compilateur indépendant pour une nouvelle fonction.
    /// Le nouveau compilateur partage les globales avec son parent et conserve
    /// une référence vers le contexte englobant afin de résoudre les captures.
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

            current_line: 0,
            current_column: 0,
        }
    }

    // ============================================================
    //                      MAIN_COMPILER
    // ============================================================

    pub fn compile(self, statements: &[Statement]) -> Result<Function, CompileError> {
        let (function, _) = self.compile_module(statements)?;

        Ok(function)
    }

    /// Enregistre une fonction native dans la table des symboles globaux.
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

    /// Retourne une copie de la table des variables locales courantes.
    pub(crate) fn locals(&self) -> LocalTable {
        self.context.borrow().locals.clone()
    }

    /// Retourne une copie des upvalues du contexte courant.
    pub(crate) fn upvalues(&self) -> Vec<Upvalue> {
        self.context.borrow().upvalues.clone()
    }

    /// Enveloppe une erreur avec la position source courante — sauf si
    /// elle est déjà enveloppée (une erreur remontant d'une fonction
    /// imbriquée compilée séparément, voir functions.rs::compile_function,
    /// porte déjà sa position, plus précise que celle de l'appelant).
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

    pub fn compile_module(
        mut self,
        statements: &[Statement],
    ) -> Result<(Function, Vec<String>), CompileError> {
        for statement in statements {
            if let Err(error) = self.compile_statement(statement) {
                return Err(self.attach_location(error));
            }
        }

        self.emit_opcode(OpCode::Halt);

        let function = Function {
            name: "<script>".to_string(),
            arity: 0,
            chunk: self.chunk,
            upvalue_count: 0,
            upvalues: Vec::new(),
        };

        Ok((function, self.exports))
    }
}