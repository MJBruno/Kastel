use crate::application::Application;

mod application;
mod ast;
mod chunk;
mod error;
mod lexer;
mod machine;
mod parser;
mod token;
mod value;
mod error_value;
mod compiler;

fn main() {
    Application::run();
}
