use crate::application::Application;

mod error;
mod lexer;
mod token;
mod application;

fn main() {
  Application::run();
}
