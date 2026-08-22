#[allow(unused_imports)]
use crate::{
    compiler::Compiler, error::RuntimeError, lexer::Lexer, machine::VirtualMachine, parser::Parser,
};

use std::{
    env, fs, io::{self, Write}, rc::Rc,
};

pub struct Application;

impl Application {
    pub fn run() {
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            let src = fs::read_to_string(&args[1]).expect("Erreur de lecture du fichier");
            execute(&src);
        } else {
            match repl() {
                Ok(run) => run,
                Err(_) => todo!(),
            }
        }
    }
}

fn execute(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_token();
    let statements = Parser::new(tokens.unwrap()).parse();
    // Compiler
    let compiler = Compiler::new();

    let function = match compiler.compile(&statements.unwrap(), 1) {
        Ok(function) => function,
        Err(error) => {
            eprintln!("Erreur de compilation : {}", error);
            return;
        }
    };

    // VM
    let function = Rc::new(function);

    let mut vm = VirtualMachine::new(function);

    if let Err(error) = vm.run() {
        eprintln!("Erreur de l'exécution du VM : {}", error);
    }
}

//Ä
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
        execute(&input);
    }
}
