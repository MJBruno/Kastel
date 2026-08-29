use crate::application::Application;

mod application;

mod bytecode;
mod compiler;
mod error;
mod frontend;
mod machine;
mod native;

mod value;
mod function;
mod closure;
mod module;


fn main() {
    Application::run();
}
