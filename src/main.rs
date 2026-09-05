use std::{process::ExitCode, time::Instant};

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
    let start = Instant::now();

    Application::run();

    let elapsed = start.elapsed();

    eprintln!(
        "\n\x1b[32mProcess success... \x1b[0m {} \x1b[32mms\x1b[0m",
        elapsed.as_secs_f64() * 1000.0
    );

    ExitCode::SUCCESS
}
