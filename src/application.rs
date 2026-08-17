use crate::lexer::Lexer;
use std::{env, fs};

pub struct Application;

impl Application {
    pub fn run() {
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            let src = fs::read_to_string(&args[1]).expect("Erreur de lecture du fichier");
            let mut lexer = Lexer::new(&src);

            match lexer.scan_tokens() {
                Ok(tokens) => {
                    for token in tokens {
                        println!("{:>15}  |  {:?}", token.lexeme(&src), token.kind);
                    }
                }

                Err(errors) => {
                    for error in errors {
                        eprintln!(
                            "main.lang:{}:{}:{}",
                            error.line, error.column, error.message
                        );
                    }
                }
            }
        } else {
            let source = r#"
     if(value> =40){
        return true;
        } 
   let value = 42 + 3.14ù123;
    "#;

            let mut lexer = Lexer::new(source);

            match lexer.scan_tokens() {
                Ok(tokens) => {
                    for token in tokens {
                        println!("{:>15}  |  {:?}", token.lexeme(source), token.kind);
                    }
                }

                Err(errors) => {
                    for error in errors {
                        eprintln!(
                            "main.lang:{}:{}:{}",
                            error.line, error.column, error.message
                        );
                    }
                }
            }
        }
    }
}
