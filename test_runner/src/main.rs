use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestKind {
    Regression,
    Error,
}

#[derive(Debug)]
struct TestCase {
    kind: TestKind,
    path: PathBuf,
}

#[derive(Debug, Default)]
struct Config {
    release: bool,
    verbose: bool,
    bless: bool,
    no_build: bool,
    keep_ansi: bool,
    filter: Option<String>,
    kind: Option<TestKind>,
    binary: Option<PathBuf>,
    root: Option<PathBuf>,
}

#[derive(Debug)]
struct TestResult {
    case: TestCase,
    passed: bool,
    duration_ms: f64,
    reason: Option<String>,
}

fn main() -> ExitCode {
    match real_main() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("test_runner: {error}");
            ExitCode::from(2)
        }
    }
}

fn real_main() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let config = parse_args(env::args_os().skip(1))?;
    let root = find_project_root(config.root.as_deref())?;

    let binary = match config.binary.clone() {
        Some(path) => absolutize(&path, &root),
        None => expected_binary_path(&root, config.release),
    };

    // Par défaut, le runner reconstruit Kastel avant la campagne.
    // Cela évite d'exécuter un ancien kastel.exe après une modification
    // du source. --no-build permet explicitement de réutiliser le binaire.
    if !config.no_build {
        println!("[runner] Construction de Kastel...");
        build_kastel(&root, config.release)?;
    } else if !binary.is_file() {
        return Err(format!(
            "--no-build demandé mais exécutable Kastel introuvable: {}",
            binary.display()
        )
        .into());
    }

    if !binary.is_file() {
        return Err(format!("exécutable Kastel introuvable: {}", binary.display()).into());
    }

    let tests = discover_tests(&root, &config)?;
    if tests.is_empty() {
        return Err("aucun test trouvé".into());
    }

    println!("Kastel Test Runner");
    println!("==================");
    println!("Root    : {}", root.display());
    println!("Binary  : {}", binary.display());
    println!("Tests   : {}", tests.len());
    if let Some(filter) = &config.filter {
        println!("Filter  : {filter}");
    }
    println!();

    let started = Instant::now();
    let mut results = Vec::with_capacity(tests.len());

    for case in tests {
        let result = run_test(&binary, &root, case, &config)?;
        print_result(&result, config.verbose);
        results.push(result);
    }

    let elapsed = started.elapsed();
    print_summary(&results, elapsed.as_secs_f64() * 1000.0);

    if results.iter().all(|result| result.passed) {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

fn parse_args<I>(args: I) -> Result<Config, Box<dyn std::error::Error>>
where
    I: IntoIterator<Item = OsString>,
{
    let mut config = Config::default();
    let mut args = args.into_iter().peekable();

    while let Some(arg) = args.next() {
        let arg = arg.to_string_lossy();
        match arg.as_ref() {
            "--release" => config.release = true,
            "--verbose" | "-v" => config.verbose = true,
            "--bless" => config.bless = true,
            "--no-build" => config.no_build = true,
            "--keep-ansi" => config.keep_ansi = true,
            "--regression" => config.kind = Some(TestKind::Regression),
            "--errors" => config.kind = Some(TestKind::Error),
            "--filter" | "-f" => {
                let value = args.next().ok_or("--filter attend une valeur")?;
                config.filter = Some(value.to_string_lossy().into_owned());
            }
            "--binary" => {
                let value = args.next().ok_or("--binary attend un chemin")?;
                config.binary = Some(PathBuf::from(value));
            }
            "--root" => {
                let value = args.next().ok_or("--root attend un chemin")?;
                config.root = Some(PathBuf::from(value));
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            value => return Err(format!("argument inconnu: {value}").into()),
        }
    }

    Ok(config)
}

fn print_help() {
    println!(
        r#"Kastel Test Runner

Usage:
  cargo run --manifest-path test_runner/Cargo.toml -- [options]

Options:
  --release             utilise target/release/kastel
  --binary <path>       exécutable Kastel explicite
  --root <path>         racine du projet Kastel explicite
  --filter <texte>      ne lance que les tests dont le chemin contient le texte
  --regression          uniquement test/regression
  --errors              uniquement test/errors
  --verbose, -v         affiche stdout/stderr en cas de succès aussi
  --bless               met à jour les .expected après vérification du succès
  --no-build             ne reconstruit pas Kastel avant les tests
  --keep-ansi           conserve les séquences ANSI dans la sortie comparée
  --help, -h            affiche cette aide

Exemples:
  cargo run --manifest-path test_runner/Cargo.toml --
  cargo run --manifest-path test_runner/Cargo.toml -- --regression
  cargo run --manifest-path test_runner/Cargo.toml -- --filter iterator
  cargo run --manifest-path test_runner/Cargo.toml -- --errors
  cargo run --manifest-path test_runner/Cargo.toml -- --release
  cargo run --manifest-path test_runner/Cargo.toml -- --no-build
"#
    );
}

fn find_project_root(explicit: Option<&Path>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(path) = explicit {
        let root = absolutize(path, Path::new("."));
        if is_project_root(&root) {
            return Ok(root);
        }
        return Err(format!("racine Kastel invalide: {}", root.display()).into());
    }

    let mut current = env::current_dir()?;
    loop {
        if is_project_root(&current) {
            return Ok(current);
        }
        if !current.pop() {
            break;
        }
    }

    Err("impossible de trouver la racine Kastel (Cargo.toml + src + test attendus)".into())
}

fn is_project_root(path: &Path) -> bool {
    path.join("Cargo.toml").is_file() && path.join("src").is_dir() && path.join("test").is_dir()
}

fn expected_binary_path(root: &Path, release: bool) -> PathBuf {
    let profile = if release { "release" } else { "debug" };
    let filename = if cfg!(windows) {
        "kastel.exe"
    } else {
        "kastel"
    };
    root.join("target").join(profile).join(filename)
}

fn build_kastel(root: &Path, release: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::new("cargo");
    command
        .current_dir(root)
        .args(["build", "--quiet", "--bin", "kastel"]);
    if release {
        command.arg("--release");
    }

    let status = command.status()?;
    if !status.success() {
        return Err(format!("cargo build a échoué avec le code {status}").into());
    }

    Ok(())
}

fn discover_tests(
    root: &Path,
    config: &Config,
) -> Result<Vec<TestCase>, Box<dyn std::error::Error>> {
    let mut tests = Vec::new();

    if config.kind != Some(TestKind::Error) {
        collect_tests(
            &root.join("test").join("regression"),
            TestKind::Regression,
            &mut tests,
        )?;
    }

    if config.kind != Some(TestKind::Regression) {
        collect_tests(
            &root.join("test").join("errors"),
            TestKind::Error,
            &mut tests,
        )?;
    }

    tests.sort_by(|left, right| left.path.cmp(&right.path));

    if let Some(filter) = &config.filter {
        let filter = filter.to_ascii_lowercase();
        tests.retain(|test| {
            test.path
                .to_string_lossy()
                .to_ascii_lowercase()
                .contains(&filter)
        });
    }

    Ok(tests)
}

fn collect_tests(dir: &Path, kind: TestKind, out: &mut Vec<TestCase>) -> io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if path.is_dir() {
            collect_tests(&path, kind, out)?;
            continue;
        }

        if path.extension().and_then(|x| x.to_str()) != Some("ks") {
            continue;
        }

        // Un fichier .ks n'est un test que s'il possède
        // son fichier .expected correspondant.
        //
        // Les modules auxiliaires importés par les tests
        // restent donc dans test/regression/, mais ne sont
        // jamais exécutés directement.
        if kind == TestKind::Regression && !path.with_extension("expected").is_file() {
            continue;
        }

        out.push(TestCase { kind, path });
    }

    Ok(())
}

fn run_test(
    binary: &Path,
    root: &Path,
    case: TestCase,
    config: &Config,
) -> Result<TestResult, Box<dyn std::error::Error>> {
    let started = Instant::now();

    let output = Command::new(binary)
        .arg(&case.path)
        .current_dir(root)
        .output()?;

    let duration_ms = started.elapsed().as_secs_f64() * 1000.0;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let actual = normalize_output(&format!("{stdout}{stderr}"), config.keep_ansi);

    let relative = relative_path(root, &case.path);

    let (passed, reason) = match case.kind {
        TestKind::Regression => {
            verify_regression(&case.path, output.status.success(), &actual, config)?
        }
        TestKind::Error => verify_error(output.status.success(), &actual),
    };

    Ok(TestResult {
        case: TestCase {
            kind: case.kind,
            path: PathBuf::from(relative),
        },
        passed,
        duration_ms,
        reason,
    })
}

fn verify_regression(
    test_path: &Path,
    success: bool,
    actual: &str,
    config: &Config,
) -> Result<(bool, Option<String>), Box<dyn std::error::Error>> {
    if !success {
        return Ok((false, Some("le processus Kastel a échoué".to_string())));
    }

    let expected_path = test_path.with_extension("expected");
    if !expected_path.is_file() {
        return Ok((
            false,
            Some(format!(".expected manquant: {}", expected_path.display())),
        ));
    }

    let expected = normalize_output(&fs::read_to_string(&expected_path)?, config.keep_ansi);
    if actual == expected {
        return Ok((true, None));
    }

    if config.bless {
        fs::write(expected_path, format!("{actual}\n"))?;
        return Ok((true, Some(".expected mis à jour avec --bless".to_string())));
    }

    Ok((false, Some(render_diff(&expected, actual))))
}

fn verify_error(success: bool, actual: &str) -> (bool, Option<String>) {
    if success {
        return (
            false,
            Some("le test devait échouer mais le processus a réussi".to_string()),
        );
    }

    let lower = actual.to_ascii_lowercase();
    let has_error_marker = lower.contains("erreur")
        || lower.contains("error")
        || lower.contains("panic")
        || lower.contains("failed");

    if has_error_marker {
        (true, None)
    } else {
        (
            false,
            Some("le processus a échoué sans message d'erreur exploitable".to_string()),
        )
    }
}

fn normalize_output(value: &str, keep_ansi: bool) -> String {
    let mut normalized = value.replace("\r\n", "\n").replace('\r', "\n");

    if !keep_ansi {
        normalized = strip_ansi(&normalized);
    }

    let mut lines = Vec::new();
    for line in normalized.lines() {
        if line.starts_with("Process finished...") {
            continue;
        }
        lines.push(line.trim_end());
    }

    lines.join("\n").trim_end().to_string()
}

fn strip_ansi(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(character) = chars.next() {
        if character == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();

            while let Some(next) = chars.next() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }

            continue;
        }

        result.push(character);
    }

    result
}

fn render_diff(expected: &str, actual: &str) -> String {
    let expected_lines: Vec<_> = expected.lines().collect();
    let actual_lines: Vec<_> = actual.lines().collect();
    let max = expected_lines.len().max(actual_lines.len());
    let mut diff = String::from("sortie différente\n");

    for index in 0..max {
        let expected_line = expected_lines.get(index).copied().unwrap_or("<EOF>");
        let actual_line = actual_lines.get(index).copied().unwrap_or("<EOF>");

        if expected_line != actual_line {
            diff.push_str(&format!(
                "ligne {}\n  attendu: {}\n  obtenu : {}",
                index + 1,
                expected_line,
                actual_line
            ));
            break;
        }
    }

    diff
}

fn print_result(result: &TestResult, verbose: bool) {
    let path = result.case.path.display();
    let kind = match result.case.kind {
        TestKind::Regression => "REG",
        TestKind::Error => "ERR",
    };

    let status = if result.passed { "PASS" } else { "FAIL" };

    println!(
        "[{kind}] {status:<4} {path:<52} {:>8.2} ms",
        result.duration_ms
    );

    if (!result.passed || verbose) && result.reason.is_some() {
        println!("      {}", result.reason.as_deref().unwrap_or_default());
    }
}

fn print_summary(results: &[TestResult], total_ms: f64) {
    let passed = results.iter().filter(|result| result.passed).count();
    let failed = results.len() - passed;

    println!();
    println!("==================");
    println!("Total  : {}", results.len());
    println!("Passed : {passed}");
    println!("Failed : {failed}");
    println!("Time   : {:.2} ms", total_ms);
    println!("==================");
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn absolutize(path: &Path, base: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }

    base.join(path)
}
