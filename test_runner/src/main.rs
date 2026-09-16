use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestKind {
    Regression,
    Error,
}

impl TestKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Regression => "REG",
            Self::Error => "ERR",
        }
    }
}

#[derive(Debug, Clone)]
struct TestCase {
    kind: TestKind,
    path: PathBuf,
    expected: PathBuf,
}

#[derive(Debug, Default)]
struct Config {
    release: bool,
    verbose: bool,
    bless: bool,
    no_build: bool,
    keep_ansi: bool,
    list_only: bool,
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
    exit_code: Option<i32>,
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

    println!("Kastel Test Runner");
    println!("==================");
    println!("Root   : {}", root.display());

    let tests = discover_tests(&root, &config)?;

    if tests.is_empty() {
        return Err("aucun test trouvé".into());
    }

    println!("Tests  : {}", tests.len());

    if let Some(filter) = &config.filter {
        println!("Filter : {filter}");
    }

    if let Some(kind) = config.kind {
        println!("Kind   : {}", kind.as_str());
    }

    println!();

    if config.list_only {
        list_tests(&root, &tests);
        return Ok(ExitCode::SUCCESS);
    }

    if !config.no_build {
        println!("[runner] Construction de Kastel...");
        build_kastel(&root, config.release)?;
        println!();
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

    println!("Binary : {}", binary.display());
    println!();

    let started = Instant::now();
    let mut results = Vec::with_capacity(tests.len());

    for case in tests {
        let result = run_test(&binary, &root, case, &config)?;
        print_result(&result, config.verbose);
        results.push(result);
    }

    let total_ms = started.elapsed().as_secs_f64() * 1000.0;

    print_summary(&results, total_ms);

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

            "--verbose" | "-v" => {
                config.verbose = true;
            }

            "--bless" => {
                config.bless = true;
            }

            "--no-build" => {
                config.no_build = true;
            }

            "--keep-ansi" => {
                config.keep_ansi = true;
            }

            "--list" => {
                config.list_only = true;
            }

            "--regression" => {
                config.kind = Some(TestKind::Regression);
            }

            "--errors" => {
                config.kind = Some(TestKind::Error);
            }

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
                return Ok(Config::default());
            }

            value => {
                return Err(format!("argument inconnu: {value}").into());
            }
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
  --release             construit Kastel en release
  --binary <path>       utilise un exécutable Kastel explicite
  --root <path>         racine du projet Kastel explicite
  --filter <texte>      filtre par chemin ou nom
  --regression          uniquement les tests de régression
  --errors              uniquement les tests d'erreur
  --verbose, -v         affiche la sortie des tests réussis
  --bless               met à jour les fichiers .expected
  --no-build            ne reconstruit pas Kastel
  --keep-ansi           conserve les séquences ANSI
  --list                affiche les tests sans les exécuter
  --help, -h            affiche cette aide

Exemples:

  Tous les tests:
    cargo run --manifest-path test_runner/Cargo.toml --

  VM uniquement:
    cargo run --manifest-path test_runner/Cargo.toml -- --filter vm

  Classes:
    cargo run --manifest-path test_runner/Cargo.toml -- --filter classes

  Modules:
    cargo run --manifest-path test_runner/Cargo.toml -- --filter modules

  Erreurs:
    cargo run --manifest-path test_runner/Cargo.toml -- --errors

  Aucun rebuild:
    cargo run --manifest-path test_runner/Cargo.toml -- --no-build

  Mettre à jour les expected:
    cargo run --manifest-path test_runner/Cargo.toml -- --bless

  Liste:
    cargo run --manifest-path test_runner/Cargo.toml -- --list
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

    Err("impossible de trouver la racine Kastel (Cargo.toml + src/ + tests/ attendus)".into())
}

fn is_project_root(path: &Path) -> bool {
    path.join("Cargo.toml").is_file() && path.join("src").is_dir() && path.join("tests").is_dir()
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
    let tests_root = root.join("tests");

    let mut tests = Vec::new();

    collect_tests(&tests_root, &mut tests)?;

    tests.sort_by(|left, right| left.path.cmp(&right.path));

    if let Some(kind) = config.kind {
        tests.retain(|test| test.kind == kind);
    }

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

fn collect_tests(dir: &Path, out: &mut Vec<TestCase>) -> io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if path.is_dir() {
            collect_tests(&path, out)?;
            continue;
        }

        if path.extension().and_then(|x| x.to_str()) != Some("ks") {
            continue;
        }

        /*
         * Convention :
         *
         *   foo.ks
         *   foo.expected
         *
         * constitue un test.
         *
         * Les fichiers .ks sans .expected sont considérés
         * comme des fichiers auxiliaires de modules/fixtures.
         */
        let expected = path.with_extension("expected");

        if !expected.is_file() {
            continue;
        }

        let kind = detect_test_kind(&path);

        out.push(TestCase {
            kind,
            path,
            expected,
        });
    }

    Ok(())
}

fn detect_test_kind(path: &Path) -> TestKind {
    let filename = path
        .file_stem()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    /*
     * Convention simple et stable :
     *
     * errors.ks
     * error.ks
     * *_error.ks
     * *_errors.ks
     *
     * sont des tests qui doivent échouer.
     */
    if filename == "error"
        || filename == "errors"
        || filename.ends_with("_error")
        || filename.ends_with("_errors")
    {
        TestKind::Error
    } else {
        TestKind::Regression
    }
}

fn list_tests(root: &Path, tests: &[TestCase]) {
    for test in tests {
        let relative = relative_path(root, &test.path);

        println!("[{}] {}", test.kind.as_str(), relative);
    }
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

    /*
     * stderr puis stdout ne sont pas forcément garantis dans
     * l'ordre temporel avec output(), mais on conserve ici le
     * comportement du runner historique.
     */
    let combined = format!("{stdout}{stderr}");

    let actual = normalize_output(&combined, config.keep_ansi);

    let (passed, reason) = match case.kind {
        TestKind::Regression => verify_regression(
            &case.expected,
            output.status.success(),
            output.status.code(),
            &actual,
            config,
        )?,

        TestKind::Error => verify_error(
            &case.expected,
            output.status.success(),
            output.status.code(),
            &actual,
            config,
        )?,
    };

    Ok(TestResult {
        case: TestCase {
            kind: case.kind,
            path: PathBuf::from(relative_path(root, &case.path)),
            expected: case.expected,
        },
        passed,
        duration_ms,
        reason,
        exit_code: output.status.code(),
    })
}

fn verify_regression(
    expected_path: &Path,
    success: bool,
    exit_code: Option<i32>,
    actual: &str,
    config: &Config,
) -> Result<(bool, Option<String>), Box<dyn std::error::Error>> {
    if !success {
        return Ok((
            false,
            Some(format!(
                "Kastel a échoué, code de sortie: {}",
                format_exit_code(exit_code)
            )),
        ));
    }

    verify_expected_output(expected_path, actual, config)
}

fn verify_error(
    expected_path: &Path,
    success: bool,
    exit_code: Option<i32>,
    actual: &str,
    config: &Config,
) -> Result<(bool, Option<String>), Box<dyn std::error::Error>> {
    if success {
        return Ok((
            false,
            Some("le test devait échouer mais Kastel a réussi".to_string()),
        ));
    }

    /*
     * Pour les tests d'erreur, on ne se contente plus de chercher
     * "error" ou "panic".
     *
     * La sortie complète doit correspondre au .expected.
     *
     * Cela permet de verrouiller :
     *
     *   main.ks:2:10
     *   Runtime error...
     *   Invalid function...
     *
     * exactement.
     */
    let result = verify_expected_output(expected_path, actual, config)?;

    if result.0 {
        Ok((
            true,
            Some(format!(
                "échec attendu, code de sortie: {}",
                format_exit_code(exit_code)
            )),
        ))
    } else {
        Ok(result)
    }
}

fn verify_expected_output(
    expected_path: &Path,
    actual: &str,
    config: &Config,
) -> Result<(bool, Option<String>), Box<dyn std::error::Error>> {
    if !expected_path.is_file() {
        return Ok((
            false,
            Some(format!(".expected manquant: {}", expected_path.display())),
        ));
    }

    let expected = normalize_output(&fs::read_to_string(expected_path)?, config.keep_ansi);

    if actual == expected {
        return Ok((true, None));
    }

    if config.bless {
        let content = if actual.is_empty() {
            String::new()
        } else {
            format!("{actual}\n")
        };

        fs::write(expected_path, content)?;

        return Ok((
            true,
            Some("fichier .expected mis à jour avec --bless".to_string()),
        ));
    }

    Ok((false, Some(render_diff(&expected, actual))))
}

fn format_exit_code(code: Option<i32>) -> String {
    match code {
        Some(code) => code.to_string(),
        None => "signal".to_string(),
    }
}

fn normalize_output(value: &str, keep_ansi: bool) -> String {
    let mut normalized = value.replace("\r\n", "\n").replace('\r', "\n");

    if !keep_ansi {
        normalized = strip_ansi(&normalized);
    }

    let mut lines = Vec::new();

    for line in normalized.lines() {
        /*
         * Ignore les messages éventuellement ajoutés
         * par certains environnements d'exécution.
         */
        if line.starts_with("Process finished...") || line.starts_with("Program finished...") {
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

    let mut diff = String::new();

    diff.push_str("sortie différente\n");

    for index in 0..max {
        let expected_line = expected_lines.get(index).copied().unwrap_or("<EOF>");

        let actual_line = actual_lines.get(index).copied().unwrap_or("<EOF>");

        if expected_line != actual_line {
            diff.push_str(&format!("ligne {}\n", index + 1));

            diff.push_str(&format!("  attendu: {}\n", expected_line));

            diff.push_str(&format!("  obtenu : {}\n", actual_line));

            /*
             * Ajouter les lignes suivantes donne un contexte
             * utile sans transformer la sortie en énorme diff.
             */
            let context_end = (index + 3).min(max);

            for context_index in index + 1..context_end {
                let expected_context = expected_lines
                    .get(context_index)
                    .copied()
                    .unwrap_or("<EOF>");

                let actual_context = actual_lines.get(context_index).copied().unwrap_or("<EOF>");

                if expected_context == actual_context {
                    diff.push_str(&format!(
                        "  contexte {}: {}\n",
                        context_index + 1,
                        actual_context
                    ));
                } else {
                    diff.push_str(&format!(
                        "  suivant {} attendu: {}\n",
                        context_index + 1,
                        expected_context
                    ));

                    diff.push_str(&format!(
                        "  suivant {} obtenu : {}\n",
                        context_index + 1,
                        actual_context
                    ));
                }
            }

            break;
        }
    }

    if expected.is_empty() && actual.is_empty() {
        diff.push_str("les deux sorties sont vides");
    }

    diff
}

fn print_result(result: &TestResult, verbose: bool) {
    let path = result.case.path.display();

    let status = if result.passed { "PASS" } else { "FAIL" };

    println!(
        "[{}] {:<4} {:<52} {:>8.2} ms",
        result.case.kind.as_str(),
        status,
        path,
        result.duration_ms
    );

    if (!result.passed || verbose) && result.reason.is_some() {
        println!("      {}", result.reason.as_deref().unwrap_or_default());
    }

    if verbose && result.exit_code.is_some() {
        println!("      exit code: {}", format_exit_code(result.exit_code));
    }
}

fn print_summary(results: &[TestResult], total_ms: f64) {
    let total = results.len();

    let passed = results.iter().filter(|result| result.passed).count();

    let failed = total - passed;

    let regression_total = results
        .iter()
        .filter(|result| result.case.kind == TestKind::Regression)
        .count();

    let regression_passed = results
        .iter()
        .filter(|result| result.case.kind == TestKind::Regression && result.passed)
        .count();

    let error_total = results
        .iter()
        .filter(|result| result.case.kind == TestKind::Error)
        .count();

    let error_passed = results
        .iter()
        .filter(|result| result.case.kind == TestKind::Error && result.passed)
        .count();

    println!();
    println!("==================");
    println!("Total       : {total}");
    println!("Passed      : {passed}");
    println!("Failed      : {failed}");
    println!();
    println!("Regression  : {regression_passed}/{regression_total}");
    println!("Errors      : {error_passed}/{error_total}");
    println!();
    println!("Time        : {:.2} ms", total_ms);
    println!("==================");

    if failed > 0 {
        println!();
        println!("Tests échoués:");

        for result in results.iter().filter(|x| !x.passed) {
            println!("  - {}", result.case.path.display());
        }
    }

    let _ = io::stdout().flush();
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
