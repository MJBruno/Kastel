use crate::app::application::Application;
mod app;
mod bytecode;
mod compile;
mod error;
mod frontend;
mod module;
mod runtime;
mod vm;

fn main() {
    Application::run();
}
