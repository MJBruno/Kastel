use std::{env, fs, path::PathBuf, process::ExitCode, rc::Rc};

use crate::{
    compiler::{
        compiler::Compiler, module_types::ModuleTypeLoader, type_checker::TypeCheckContext,
    },
    error::kastel_error::KastelError,
    error::runtime_error::RuntimeError,
    frontend::{lexer::lexer::Lexer, parser::Parser},
    stdlib::execute_native,
    vm::machine::VirtualMachine,
};

use super::repl;

pub struct Application;

impl Application {
    pub fn run() -> ExitCode {
        let mut args = env::args();
        let _program = args.next();

        let Some(argument) = args.next() else {
            repl::run();
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

/// Exécute un fichier source complet (contrairement au REPL — voir
/// `repl.rs` — qui compile chaque ligne séparément).
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
