//! Boucle interactive (`kastel` sans argument).
//!
//! Séparé de `application.rs` : ce fichier ne connaît que la session REPL
//! (`ReplSession`) et la boucle de lecture/exécution/affichage (`run`),
//! alors que `application.rs` reste le point d'entrée du binaire (parsing
//! des arguments, exécution d'un fichier).
//!
//! Contrairement à un fichier exécuté par `kastel <fichier.ks>`, le REPL
//! compile CHAQUE ligne séparément (voir `ReplSession::execute`) : les
//! globales de la VM et du compilateur sont conservées d'une ligne à
//! l'autre (une fonction ou une variable déclarée reste visible ensuite),
//! mais le vérificateur de types, lui, repart de zéro à chaque ligne — il
//! ne mémorise pas les types déduits des lignes précédentes.

use std::{
    cell::RefCell,
    collections::HashMap,
    env,
    io::{self, Write},
    path::PathBuf,
    rc::Rc,
};

use crate::{
    compiler::{
        compiler::Compiler, module_types::ModuleTypeLoader, type_checker::TypeCheckContext,
        variables::Global,
    },
    error::kastel_error::KastelError,
    error::runtime_error::RuntimeError,
    frontend::{lexer::lexer::Lexer, parser::Parser},
    module::{module::ModuleLoader, resolver::ModuleResolver},
    runtime::value::Value,
    stdlib::execute_native,
    vm::machine::VirtualMachine,
};

/// État d'une session REPL : globales du compilateur et de la VM,
/// historique, et contexte de résolution des imports.
struct ReplSession {
    compiler_globals: Rc<RefCell<HashMap<String, Global>>>,
    vm: VirtualMachine,
    history: Vec<String>,

    /// Fichier fictif du REPL, dans le répertoire courant : les imports
    /// (`import utils;`, `from lib.mod import X;`) se résolvent à partir de
    /// ce répertoire, comme pour un fichier source placé là.
    repl_path: PathBuf,

    /// Chargeur de types partagé par toutes les lignes (cache des modules
    /// déjà analysés).
    type_loader: Rc<ModuleTypeLoader>,
}

impl ReplSession {
    fn new() -> Result<Self, KastelError> {
        let mut compiler = Compiler::new();
        execute_native(&mut compiler);

        let compiler_globals = Rc::clone(&compiler.globals);

        let function = Rc::new(compiler.compile(&[]).map_err(KastelError::from)?);

        let project_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let repl_path = project_root.join("<repl>");

        let resolver = ModuleResolver::new(project_root);
        let type_loader = Rc::new(ModuleTypeLoader::new(resolver.clone()));

        let vm = VirtualMachine::new_with_loader(
            function,
            Some(repl_path.clone()),
            ModuleLoader::with_resolver(resolver),
        );

        Ok(Self {
            compiler_globals,
            vm,
            history: Vec::new(),
            repl_path,
            type_loader,
        })
    }

    fn execute(&mut self, source: &str) -> Result<Option<Value>, KastelError> {
        let tokens = Lexer::new(source.to_string())
            .scan_token()
            .map_err(KastelError::from)?;

        let mut parser = Parser::new(tokens);

        let statements = parser.parse().map_err(KastelError::from)?;

        if statements.is_empty() {
            return Ok(None);
        }

        let compiler = Compiler::new_with_globals(Rc::clone(&self.compiler_globals));

        let context = TypeCheckContext::new(self.repl_path.clone(), Rc::clone(&self.type_loader));

        let function = Rc::new(
            compiler
                .compile_repl_with_context(&statements, context)
                .map_err(KastelError::from)?,
        );

        let result = self
            .vm
            .execute_repl(function)
            .map_err(|error| RuntimeError::WithLocation {
                line: self.vm.current_line,
                column: self.vm.current_column,
                source: Box::new(error),
            })
            .map_err(KastelError::from)?;

        self.history.push(source.to_string());

        Ok(result)
    }

    fn reset(&mut self) -> Result<(), KastelError> {
        *self = Self::new()?;
        Ok(())
    }

    fn clear_screen() {
        print!("\x1B[2J\x1B[H");
        let _ = io::stdout().flush();
    }

    fn print_help() {
        println!("Commandes Kastel REPL:");
        println!("  :help       Afficher cette aide");
        println!("  :history    Afficher l'historique");
        println!("  :clear      Effacer l'écran");
        println!("  :reset      Réinitialiser la session");
        println!("  :quit       Quitter");
        println!("  :exit       Quitter");
    }
}

/// Point d'entrée du REPL, appelé par `Application::run` quand `kastel` est
/// lancé sans argument.
pub fn run() {
    println!("Kastel REPL");
    println!("Tapez :help pour l'aide.");

    let mut session = match ReplSession::new() {
        Ok(session) => session,
        Err(error) => {
            eprintln!("Erreur d'initialisation du REPL : {error}");
            return;
        }
    };

    let mut buffer = String::new();

    loop {
        if buffer.is_empty() {
            print!(">>> ");
        } else {
            print!("... ");
        }

        if io::stdout().flush().is_err() {
            break;
        }

        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {}
            Err(error) => {
                eprintln!("Erreur de lecture : {error}");
                break;
            }
        }

        let line = input.trim_end_matches(['\r', '\n']);

        if buffer.is_empty() && line.starts_with(':') {
            match line.trim() {
                ":help" => ReplSession::print_help(),
                ":clear" => ReplSession::clear_screen(),

                ":history" => {
                    for (index, item) in session.history.iter().enumerate() {
                        println!("{:>4}  {}", index + 1, item);
                    }
                }

                ":reset" => match session.reset() {
                    Ok(()) => println!("Session réinitialisée."),
                    Err(error) => {
                        eprintln!("Erreur de réinitialisation : {error}")
                    }
                },

                ":quit" | ":exit" => break,

                command => {
                    eprintln!("Commande inconnue : {command}");
                    eprintln!("Tapez :help.");
                }
            }

            continue;
        }

        buffer.push_str(line);
        buffer.push('\n');

        if !input_complete(&buffer) {
            continue;
        }

        match session.execute(&buffer) {
            Ok(Some(value)) => println!("{value}"),
            Ok(None) => {}
            Err(error) => eprintln!("{}", error.render(&buffer, "<repl>")),
        }

        buffer.clear();
    }
}

/// `false` tant qu'une chaîne, un bloc, un tableau ou un appel est encore
/// ouvert : le REPL affiche alors `...` et attend la suite.
fn input_complete(source: &str) -> bool {
    let mut braces = 0usize;
    let mut brackets = 0usize;
    let mut parentheses = 0usize;

    let mut delimiter: Option<char> = None;
    let mut escaped = false;

    for character in source.chars() {
        if let Some(active_delimiter) = delimiter {
            if escaped {
                escaped = false;
                continue;
            }

            if character == '\\' {
                escaped = true;
                continue;
            }

            if character == active_delimiter {
                delimiter = None;
            }

            continue;
        }

        match character {
            '"' | '\'' => delimiter = Some(character),

            '{' => braces += 1,
            '}' => braces = braces.saturating_sub(1),

            '[' => brackets += 1,
            ']' => brackets = brackets.saturating_sub(1),

            '(' => parentheses += 1,
            ')' => parentheses = parentheses.saturating_sub(1),

            _ => {}
        }
    }

    delimiter.is_none() && braces == 0 && brackets == 0 && parentheses == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_line(session: &mut ReplSession, source: &str) -> Option<Value> {
        session
            .execute(source)
            .map_err(|error| error.to_string())
            .unwrap_or_else(|message| panic!("`{source}` a échoué : {message}"))
    }

    #[test]
    fn imports_work_in_the_repl() {
        let mut session = ReplSession::new().expect("session REPL");

        // Import qualifié d'un module de la bibliothèque standard.
        run_line(&mut session, "import std.math;");

        assert!(matches!(
            run_line(&mut session, "math.gcd(48, 18)"),
            Some(Value::Integer(6))
        ));

        // Import direct d'une classe, puis usage sur une ligne suivante.
        run_line(&mut session, "import std.math.Complexe;");

        assert!(matches!(
            run_line(&mut session, "new Complexe(3, 4).magnitude()"),
            Some(Value::Float(value)) if value == 5.0
        ));
    }

    #[test]
    fn a_function_can_be_redefined_in_the_repl() {
        // Le REPL EST le seul endroit où redéfinir une fonction déjà
        // déclarée est permis (un fichier source refuse le doublon de
        // même arité) : chaque ligne est un nouveau fragment de
        // compilation, la redéfinition remplace la globale existante.
        let mut session = ReplSession::new().expect("session REPL");

        run_line(&mut session, "func double(x) { return x * 2; }");
        assert!(matches!(
            run_line(&mut session, "double(21)"),
            Some(Value::Integer(42))
        ));

        run_line(&mut session, "func double(x) { return x * 3; }");
        assert!(matches!(
            run_line(&mut session, "double(21)"),
            Some(Value::Integer(63))
        ));
    }

    #[test]
    fn input_completeness_tracks_braces_brackets_parens_and_strings() {
        assert!(!input_complete("func f() {\n"));
        assert!(input_complete("func f() {}\n"));
        assert!(!input_complete("let a = [1, 2\n"));
        assert!(input_complete("let a = [1, 2]\n"));
        assert!(!input_complete("let s = \"{ not a block\n"));
        assert!(input_complete("let s = \"{ not a block\"\n"));
    }
}
