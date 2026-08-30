use crate::compile::compiler::Compiler;
use crate::error::kastel_error::KastelError;
use crate::frontend::lexer::Lexer;
use crate::frontend::parser::Parser;
use crate::runtime::native::execute_native;
use crate::vm::machine::VirtualMachine;

use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
    rc::Rc,
};

pub struct Application;

impl Application {
    pub fn run() -> ExitCode {
        let args: Vec<String> = env::args().collect();

        if args.len() > 1 {
            let path = match PathBuf::from(&args[1]).canonicalize() {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("Erreur : impossible de résoudre '{}': {}", args[1], error);
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

            match execute(&src, Some(path)) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::FAILURE
                }
            }
        } else {
            repl();
            ExitCode::SUCCESS
        }
    }
}

/// Exécute le pipeline complet Lexer → Parser → Compiler → VM sur une
/// source donnée. Chaque étape peut échouer avec son propre type d'erreur ;
/// `KastelError` unifie le tout pour que `?` fonctionne de bout en bout.
fn execute(source: &str, module_path: Option<PathBuf>) -> Result<(), KastelError> {
    let tokens = Lexer::new(source.to_string()).scan_token()?;
    let statements = Parser::new(tokens).parse()?;

    let mut compiler = Compiler::new();
    execute_native(&mut compiler);

    let function = Rc::new(compiler.compile(&statements)?);

    VirtualMachine::new(function, module_path).run()?;

    Ok(())
}

fn repl() {
    println!("Crafted by nova.org, Madagascar: 2026 – 2027 ");

    loop {
        print!("[Nova]👉  ");

        if io::stdout().flush().is_err() {
            break;
        }

        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            // read_line retourne Ok(0) en cas d'EOF (Ctrl+D / Ctrl+Z) :
            // sans ce cas, la boucle précédente tournait à vide indéfiniment
            // au lieu de quitter proprement.
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }

        if input.trim().is_empty() {
            continue;
        }

        if let Err(error) = execute(&input, None) {
            eprintln!("{error}");
        }
    }
}