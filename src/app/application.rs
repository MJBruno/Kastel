use std::{
    cell::RefCell,
    collections::HashMap,
    env, fs,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
    rc::Rc,
};

use crate::{
    compiler::{
        compiler::Compiler,
        module_types::ModuleTypeLoader,
        type_checker::TypeCheckContext,
        variables::Global,
    },
    module::{module::ModuleLoader, resolver::ModuleResolver},
    error::kastel_error::KastelError,
    error::runtime_error::RuntimeError,
    frontend::{lexer::lexer::Lexer, parser::Parser},
    runtime::value::Value,
    stdlib::execute_native,
    vm::machine::VirtualMachine,
};

pub struct Application;

impl Application {
    pub fn run() -> ExitCode {
        let mut args = env::args();
        let _program = args.next();

        let Some(argument) = args.next() else {
            repl();
            return ExitCode::SUCCESS;
        };

        match argument.as_str() {
            "--version" | "-v" => {
                println!("Kastel {}", env!("CARGO_PKG_VERSION"));
                ExitCode::SUCCESS
            }

            "--help" | "-h" => {
                print_help();
                ExitCode::SUCCESS
            }

            _ if argument.starts_with('-') => {
                eprintln!("Option inconnue : {argument}");
                eprintln!("Utilisez 'kastel --help' pour afficher l'aide.");
                ExitCode::FAILURE
            }

            path => {
                let path = match PathBuf::from(path).canonicalize() {
                    Ok(path) => path,
                    Err(error) => {
                        eprintln!("Erreur : impossible de résoudre '{}': {}", path, error);
                        return ExitCode::FAILURE;
                    }
                };

                let src = match fs::read_to_string(&path) {
                    Ok(src) => src,
                    Err(error) => {
                        eprintln!(
                            "Erreur de lecture du fichier '{}': {}",
                            path.display(),
                            error
                        );
                        return ExitCode::FAILURE;
                    }
                };

                match execute(&src, Some(path.clone())) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(error) => {
                        let file_name = path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("<unknown>");

                        eprintln!("{}", error.render(&src, file_name));
                        ExitCode::FAILURE
                    }
                }
            }
        }
    }
}
fn print_help() {
    println!("Kastel {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Usage:");
    println!("  kastel <file.ks>    Exécuter un programme Kastel");
    println!("  kastel              Démarrer le REPL");
    println!("  kastel --version    Afficher la version");
    println!("  kastel --help       Afficher cette aide");
}
fn execute(source: &str, module_path: Option<PathBuf>) -> Result<(), KastelError> {
    let tokens = Lexer::new(source.to_string())
        .scan_token()
        .map_err(KastelError::from)?;

    let mut parser = Parser::new(tokens);

    let statements = parser.parse().map_err(KastelError::from)?;

    let mut compiler = Compiler::new();
    execute_native(&mut compiler);

    let (function, runtime_loader) = if let Some(module_path) = &module_path {
        // Le resolver utilisé par le TypeChecker et celui utilisé par la VM
        // doivent être identiques. Sinon un import peut être accepté pendant
        // l'analyse statique puis résolu vers un autre chemin à l'exécution.
        let project_root = env::current_dir().unwrap_or_else(|_| {
            module_path
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
        });

        let resolver = crate::module::resolver::ModuleResolver::new(project_root);
        let type_loader = Rc::new(ModuleTypeLoader::new(resolver.clone()));
        let context = TypeCheckContext::new(module_path.clone(), type_loader);
        let function = Rc::new(
            compiler
                .compile_with_context(&statements, context)
                .map_err(KastelError::from)?,
        );

        let runtime_loader = crate::module::module::ModuleLoader::with_resolver(resolver);

        (function, runtime_loader)
    } else {
        let function = Rc::new(compiler.compile(&statements).map_err(KastelError::from)?);
        let runtime_loader = crate::module::module::ModuleLoader::new(
            env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        );
        (function, runtime_loader)
    };

    let mut vm = VirtualMachine::new_with_loader(function, module_path, runtime_loader);

    if let Err(error) = vm.run() {
        return Err(RuntimeError::WithLocation {
            line: vm.current_line,
            column: vm.current_column,
            source: Box::new(error),
        }
        .into());
    }

    Ok(())
}

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

fn repl() {
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
mod repl_tests {
    use super::*;

    fn run(session: &mut ReplSession, source: &str) -> Option<Value> {
        session
            .execute(source)
            .map_err(|error| error.to_string())
            .unwrap_or_else(|message| panic!("`{source}` a échoué : {message}"))
    }

    #[test]
    fn imports_work_in_the_repl() {
        let mut session = ReplSession::new().expect("session REPL");

        // Import qualifié d'un module de la bibliothèque standard.
        run(&mut session, "import std.math;");

        assert!(matches!(
            run(&mut session, "math.gcd(48, 18)"),
            Some(Value::Integer(6))
        ));

        // Import direct d'une classe, puis usage sur une ligne suivante.
        run(&mut session, "import std.math.Complexe;");

        assert!(matches!(
            run(&mut session, "new Complexe(3, 4).magnitude()"),
            Some(Value::Float(value)) if value == 5.0
        ));
    }
}

