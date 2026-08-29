#[allow(unused_imports)]
use crate::{
    compiler::Compiler, error::RuntimeError, lexer::Lexer, machine::VirtualMachine,
    native::execute_native, parser::Parser,
};

use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
    rc::Rc,
};

pub struct Application;

impl Application {
    pub fn run() {
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            let path = match std::path::PathBuf::from(&args[1]).canonicalize() {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("Erreur : impossible de résoudre '{}': {}", args[1], error);
                    return;
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
                    return;
                }
            };

            execute(&src, Some(path));
        } else {
            match repl() {
                Ok(run) => run,
                Err(_) => todo!(),
            }
        }
    }
}

fn execute(source: &str, module_path: Option<PathBuf>) {
    let mut lexer = Lexer::new(source.to_string());

    let tokens = match lexer.scan_token() {
        Ok(tokens) => tokens,
        Err(error) => {
            eprintln!("Erreur lexicale : {:?}", error);
            return;
        }
    };

    let statements = match Parser::new(tokens).parse() {
        Ok(statements) => statements,
        Err(error) => {
            eprintln!("Erreur de parsing : {:?}", error);
            return;
        }
    };

    let mut compiler = Compiler::new();
    execute_native(&mut compiler);

    let function = match compiler.compile(&statements) {
        Ok(function) => function,
        Err(error) => {
            eprintln!("Erreur de compilation : {}", error);
            return;
        }
    };

    let function = Rc::new(function);
    let mut vm = VirtualMachine::new(function, module_path);

    if let Err(error) = vm.run() {
        eprintln!("Erreur de l'exécution du VM : {}", error);
    }
}
fn repl() -> Result<(), RuntimeError> {
    println!("Crafted by nova.org, Madagascar: 2026 – 2027 ");
    loop {
        print!("[Nova]👉  ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break Ok(());
        }
        if input.trim().is_empty() {
            continue;
        }
     execute(&input, None);
    }
}
