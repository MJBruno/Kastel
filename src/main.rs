use crate::application::Application;

mod application;
mod ast;
mod bytecode;
mod compiler;
mod error;
mod lexer;
mod machine;
mod native;
mod parser;
mod token;
mod value;
mod function;
mod closure;
mod module;


fn main() {
    Application::run();
}
