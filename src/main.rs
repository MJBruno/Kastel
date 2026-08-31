use crate::app::application::Application;
mod app;
mod bytecode;
mod compiler;
mod error;
mod frontend;
mod module;
mod runtime;
mod vm;

fn main() -> std::process::ExitCode {
    Application::run()
}