use std::time::Instant;

use crate::app::application::Application;
mod app;
mod bytecode;
mod compiler;
mod error;
mod frontend;
mod module;
mod runtime;
mod vm;
mod stdlib;

fn main() -> std::process::ExitCode {
    let start = Instant::now();

    let exit_code = Application::run();

    let elapsed = start.elapsed();

    eprintln!(
        "\n\x1b[32mProcess finished... \x1b[0m {} \x1b[32mms\x1b[0m",
        elapsed.as_secs_f64() * 1000.0
    );

    exit_code
}