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
mod compiler;
mod native;

fn main() {
    Application::run();
}
