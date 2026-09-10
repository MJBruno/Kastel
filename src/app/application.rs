use std::{
    cell::RefCell,
    collections::HashMap,
    env,
    fs,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
    rc::Rc,
};

use crate::{
    compiler::{
        compiler::Compiler,
        variables::Global,
    },
    error::kastel_error::KastelError,
    error::runtime_error::RuntimeError,
    frontend::{
        lexer::Lexer,
        parser::Parser,
    },
    runtime::value::Value,
    stdlib::execute_native,
    vm::machine::VirtualMachine,
};

pub struct Application;

impl Application {
    pub fn run() -> ExitCode {
        let args: Vec<String> = env::args().collect();

        if args.len() > 1 {
            let path = match PathBuf::from(&args[1]).canonicalize() {
                Ok(path) => path,
                Err(error) => {
                    eprintln!(
                        "Erreur : impossible de résoudre '{}': {}",
                        args[1],
                        error
                    );
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
                    eprintln!("Erreur dans '{}' :", path.display());
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

fn execute(
    source: &str,
    module_path: Option<PathBuf>,
) -> Result<(), KastelError> {
    let tokens = Lexer::new(source.to_string()).scan_token()?;
    let statements = Parser::new(tokens).parse()?;

    let mut compiler = Compiler::new();
    execute_native(&mut compiler);

    let function = Rc::new(compiler.compile(&statements)?);

    let mut vm = VirtualMachine::new(function, module_path);

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
}

impl ReplSession {
    fn new() -> Self {
        let mut compiler = Compiler::new();
        execute_native(&mut compiler);

        let compiler_globals = Rc::clone(&compiler.globals);

        let function = Rc::new(
            compiler
                .compile(&[])
                .expect("compilation du contexte REPL impossible"),
        );

        let vm = VirtualMachine::new(function, None);

        Self {
            compiler_globals,
            vm,
            history: Vec::new(),
        }
    }

    fn execute(
        &mut self,
        source: &str,
    ) -> Result<Option<Value>, KastelError> {
        let tokens = Lexer::new(source.to_string()).scan_token()?;
        let statements = Parser::new(tokens).parse()?;

        if statements.is_empty() {
            return Ok(None);
        }

        let compiler = Compiler::new_with_globals(
            Rc::clone(&self.compiler_globals),
        );

        let function = Rc::new(
            compiler.compile_repl(&statements)?,
        );

        let result = self.vm.execute_repl(function)?;

        self.history.push(source.to_string());

        Ok(result)
    }

    fn reset(&mut self) {
        *self = Self::new();
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

    let mut session = ReplSession::new();
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

                ":reset" => {
                    session.reset();
                    println!("Session réinitialisée.");
                }

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
            Ok(Some(value)) => {
                println!("{value}");
            }

            Ok(None) => {}

            Err(error) => {
                eprintln!("{error}");
            }
        }

        buffer.clear();
    }
}

fn input_complete(source: &str) -> bool {
    let mut braces = 0usize;
    let mut brackets = 0usize;
    let mut parentheses = 0usize;

    let mut string = false;
    let mut escaped = false;

    for character in source.chars() {
        if string {
            if escaped {
                escaped = false;
                continue;
            }

            if character == '\\' {
                escaped = true;
                continue;
            }

            if character == '"' {
                string = false;
            }

            continue;
        }

        match character {
            '"' => string = true,

            '{' => braces += 1,
            '}' => braces = braces.saturating_sub(1),

            '[' => brackets += 1,
            ']' => brackets = brackets.saturating_sub(1),

            '(' => parentheses += 1,
            ')' => parentheses = parentheses.saturating_sub(1),

            _ => {}
        }
    }

    !string
        && braces == 0
        && brackets == 0
        && parentheses == 0
}