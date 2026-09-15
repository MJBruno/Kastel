use std::time::Instant;

use kastel::app::application::Application;

fn main() -> std::process::ExitCode {
    let start = Instant::now();

    let exit_code = Application::run();

    let elapsed = start.elapsed();

    println!(
        "\x1b[32mProcess finished... \x1b[0m {:.3} \x1b[32mms\x1b[0m",
        elapsed.as_secs_f64() * 1000.0
    );

    exit_code
}