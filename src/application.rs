#[allow(unused_imports)]
use crate::{
    compiler::Compiler, error::RuntimeError, lexer::Lexer, machine::VirtualMachine, parser::Parser,
};

use std::{
    env, fs,
    io::{self, Write},
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
    let parser = Parser::new(tokens.unwrap()).parse();
    let chunk = Compiler::new().compile(&parser.unwrap(), 0);
    let mut vm = VirtualMachine::new(chunk.expect("Erreur de l'execution du Machine virtuel"));
    match vm.run() {
        Ok(ok) => ok,
        Err(e) => eprintln!("{e}"),
    }
    //     match parser {
    //     Ok(statements) => {
    //         println!("{:#?}", statements);
    //     }

    //     Err(errors) => {
    //         for error in errors {
    //             eprintln!(
    //                 "{}:{}: {}",
    //                 error.line,
    //                 error.column,
    //                 error.message
    //             );
    //         }
    //     }
    // }
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
