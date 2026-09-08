use std::{
    env,
    ffi::OsStr,
    fs,
    io,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

#[derive(Debug)]
struct TestResult {
    path: PathBuf,
    passed: bool,
    message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Check,
    Update,
    Clean,
}

fn main() -> ExitCode {
    let (mode, test_dir) = match parse_args() {
        Ok(value) => value,

        Err(error) => {
            eprintln!("Erreur : {error}");
            return ExitCode::FAILURE;
        }
    };

    if !test_dir.is_dir() {
        eprintln!(
            "Erreur : dossier de tests introuvable : {}",
            test_dir.display()
        );

        return ExitCode::FAILURE;
    }

    /*
     * CLEAN ne nécessite pas de compiler Kastel.
     */
    if mode == Mode::Clean {
        return clean_expected_files(&test_dir);
    }

    println!("Kastel Test Runner");
    println!("==================");
    println!("Dossier : {}", test_dir.display());

    match mode {
        Mode::Check => println!("Mode : CHECK"),
        Mode::Update => println!("Mode : UPDATE"),
        Mode::Clean => unreachable!(),
    }

    println!();

    /*
     * Compilation de Kastel.
     */
    let executable = match build_kastel() {
        Ok(path) => path,

        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    /*
     * Recherche uniquement dans :
     *
     * test/errors/
     * test/regression/
     *
     * Les dossiers showcase/closure/etc. sont ignorés.
     */
    let tests = match collect_tests(&test_dir) {
        Ok(tests) => tests,

        Err(error) => {
            eprintln!("Erreur lecture des tests : {error}");
            return ExitCode::FAILURE;
        }
    };

    println!("Tests trouvés : {}", tests.len());
    println!();

    if tests.is_empty() {
        println!("Aucun test automatisé trouvé.");
        return ExitCode::SUCCESS;
    }

    let mut results = Vec::with_capacity(tests.len());

    for test in tests {
        let result = run_test(&executable, &test, mode);

        if result.passed {
            println!("PASS {}", display_path(&result.path));

            if !result.message.is_empty() {
                println!("{}", indent(&result.message));
            }
        } else {
            println!("FAIL {}", display_path(&result.path));

            if !result.message.is_empty() {
                println!("{}", indent(&result.message));
            }
        }

        results.push(result);
    }

    print_summary(&results);

    if results.iter().all(|result| result.passed) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn parse_args() -> Result<(Mode, PathBuf), String> {
    let mut mode = Mode::Check;
    let mut test_dir = PathBuf::from("test");

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--update" => {
                if mode != Mode::Check {
                    return Err(
                        "--update et --clean ne peuvent pas être utilisés ensemble."
                            .to_string(),
                    );
                }

                mode = Mode::Update;
            }

            "--clean" => {
                if mode != Mode::Check {
                    return Err(
                        "--update et --clean ne peuvent pas être utilisés ensemble."
                            .to_string(),
                    );
                }

                mode = Mode::Clean;
            }

            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }

            _ => {
                test_dir = PathBuf::from(arg);
            }
        }
    }

    Ok((mode, test_dir))
}

fn print_help() {
    println!("Kastel Test Runner");
    println!();
    println!("Usage:");
    println!("  cargo run --bin test_runner");
    println!("  cargo run --bin test_runner -- --update");
    println!("  cargo run --bin test_runner -- --clean");
    println!();
    println!("Dossier spécifique:");
    println!("  cargo run --bin test_runner -- test/regression");
    println!("  cargo run --bin test_runner -- --update test/regression");
    println!("  cargo run --bin test_runner -- --clean test/regression");
}

fn build_kastel() -> Result<PathBuf, String> {
    let status = Command::new(cargo_program())
        .args(["build", "--quiet", "--bin", "kastel"])
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| {
            format!("Impossible de lancer Cargo : {error}")
        })?;

    if !status.success() {
        return Err(
            "Erreur : impossible de compiler Kastel.".to_string()
        );
    }

    let current_exe = env::current_exe().map_err(|error| {
        format!(
            "Impossible de déterminer le chemin du test runner : {error}"
        )
    })?;

    let debug_dir = current_exe.parent().ok_or_else(|| {
        "Répertoire de l'exécutable introuvable.".to_string()
    })?;

    let executable_name = if cfg!(windows) {
        "kastel.exe"
    } else {
        "kastel"
    };

    let executable = debug_dir.join(executable_name);

    if !executable.is_file() {
        return Err(format!(
            "Exécutable Kastel introuvable : {}",
            executable.display()
        ));
    }

    Ok(executable)
}

fn cargo_program() -> &'static str {
    if cfg!(windows) {
        "cargo.exe"
    } else {
        "cargo"
    }
}

/*
 * Détermine les répertoires de tests automatisés.
 *
 * Si on lance :
 *
 * cargo run --bin test_runner
 *
 * on utilise :
 *
 * test/errors
 * test/regression
 *
 * Si on lance directement :
 *
 * cargo run --bin test_runner -- test/regression
 *
 * on utilise ce dossier comme racine.
 */
fn collect_tests(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut tests = Vec::new();

    let normalized_name = dir
        .file_name()
        .and_then(|value| value.to_str());

    match normalized_name {
        Some("errors") | Some("regression") => {
            collect_tests_recursive(dir, &mut tests)?;
        }

        _ => {
            let errors_dir = dir.join("errors");
            let regression_dir = dir.join("regression");

            if errors_dir.is_dir() {
                collect_tests_recursive(
                    &errors_dir,
                    &mut tests,
                )?;
            }

            if regression_dir.is_dir() {
                collect_tests_recursive(
                    &regression_dir,
                    &mut tests,
                )?;
            }
        }
    }

    tests.sort();

    Ok(tests)
}

fn collect_tests_recursive(
    dir: &Path,
    tests: &mut Vec<PathBuf>,
) -> Result<(), io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_tests_recursive(&path, tests)?;
            continue;
        }

        if path.extension() == Some(OsStr::new("ks")) {
            tests.push(path);
        }
    }

    Ok(())
}

fn run_test(
    executable: &Path,
    test_path: &Path,
    mode: Mode,
) -> TestResult {
    let output = match Command::new(executable)
        .arg(test_path)
        .output()
    {
        Ok(output) => output,

        Err(error) => {
            return TestResult {
                path: test_path.to_path_buf(),
                passed: false,
                message: format!(
                    "Impossible d'exécuter Kastel : {error}"
                ),
            };
        }
    };

    let stdout = normalize_output(
        &String::from_utf8_lossy(&output.stdout),
    );

    let stderr = normalize_output(
        &String::from_utf8_lossy(&output.stderr),
    );

    let is_error_test = is_error_test(test_path);

    let has_kastel_error =
        stdout.contains("Erreur dans")
            || stderr.contains("Erreur dans")
            || stdout.contains("Erreur de compilation")
            || stderr.contains("Erreur de compilation")
            || stdout.contains("Erreur d'exécution")
            || stderr.contains("Erreur d'exécution")
            || stdout.contains("Erreur(s) de parsing")
            || stderr.contains("Erreur(s) de parsing");

    /*
     * ------------------------------------------------------------
     * TESTS D'ERREURS
     * ------------------------------------------------------------
     */
    if is_error_test {
        if has_kastel_error {
            return TestResult {
                path: test_path.to_path_buf(),
                passed: true,
                message: String::new(),
            };
        }

        return TestResult {
            path: test_path.to_path_buf(),
            passed: false,
            message:
                "Une erreur Kastel était attendue, mais aucune erreur n'a été détectée."
                    .to_string(),
        };
    }

    /*
     * ------------------------------------------------------------
     * TEST NORMAL
     * ------------------------------------------------------------
     */

    if has_kastel_error {
        return TestResult {
            path: test_path.to_path_buf(),
            passed: false,
            message: format!(
                "Erreur Kastel inattendue.\n\nstdout:\n{}\n\nstderr:\n{}",
                stdout,
                stderr
            ),
        };
    }

    let expected_path = test_path.with_extension("expected");

    /*
     * ------------------------------------------------------------
     * UPDATE
     * ------------------------------------------------------------
     *
     * Création ou remplacement du .expected.
     */
    if mode == Mode::Update {
        if let Err(error) = fs::write(&expected_path, &stdout) {
            return TestResult {
                path: test_path.to_path_buf(),
                passed: false,
                message: format!(
                    "Impossible d'écrire {} : {}",
                    expected_path.display(),
                    error
                ),
            };
        }

        return TestResult {
            path: test_path.to_path_buf(),
            passed: true,
            message: format!(
                "Sortie attendue générée : {}",
                display_path(&expected_path)
            ),
        };
    }

    /*
     * ------------------------------------------------------------
     * CHECK
     * ------------------------------------------------------------
     */

    if !expected_path.is_file() {
        return TestResult {
            path: test_path.to_path_buf(),
            passed: false,
            message: format!(
                "Fichier attendu manquant : {}",
                display_path(&expected_path)
            ),
        };
    }

    let expected = match fs::read_to_string(&expected_path) {
        Ok(content) => normalize_output(&content),

        Err(error) => {
            return TestResult {
                path: test_path.to_path_buf(),
                passed: false,
                message: format!(
                    "Impossible de lire {} : {}",
                    display_path(&expected_path),
                    error
                ),
            };
        }
    };

    /*
     * Comparaison exacte.
     */
    if stdout != expected {
        return TestResult {
            path: test_path.to_path_buf(),
            passed: false,
            message: format!(
                "Sortie différente de la sortie attendue.\n\n\
                 Attendu:\n{}\n\n\
                 Obtenu:\n{}",
                expected,
                stdout
            ),
        };
    }

    TestResult {
        path: test_path.to_path_buf(),
        passed: true,
        message: String::new(),
    }
}

fn is_error_test(path: &Path) -> bool {
    path.components().any(|component| {
        component.as_os_str() == OsStr::new("errors")
    })
}

fn normalize_output(value: &str) -> String {
    value
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim_end()
        .to_string()
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn indent(value: &str) -> String {
    value
        .lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn print_summary(results: &[TestResult]) {
    let passed = results
        .iter()
        .filter(|result| result.passed)
        .count();

    let failed = results.len() - passed;

    println!();
    println!("==================");
    println!("Total  : {}", results.len());
    println!("Passed : {passed}");
    println!("Failed : {failed}");
    println!("==================");
}

/*
 * ------------------------------------------------------------
 * CLEAN
 * ------------------------------------------------------------
 *
 * Supprime uniquement les .expected dans les dossiers de tests
 * sélectionnés par le runner.
 */
fn clean_expected_files(dir: &Path) -> ExitCode {
    println!("Kastel Test Runner");
    println!("==================");
    println!("Mode : CLEAN");
    println!("Dossier : {}", dir.display());
    println!();

    let directories = match clean_directories(dir) {
        Ok(value) => value,

        Err(error) => {
            eprintln!(
                "Erreur lecture des dossiers de tests : {error}"
            );

            return ExitCode::FAILURE;
        }
    };

    let mut removed = 0usize;
    let mut errors = 0usize;

    for directory in directories {
        if let Err(error) = clean_expected_recursive(
            &directory,
            &mut removed,
            &mut errors,
        ) {
            eprintln!(
                "Erreur dans {} : {}",
                display_path(&directory),
                error
            );

            errors += 1;
        }
    }

    println!();
    println!("Fichiers .expected supprimés : {removed}");

    if errors == 0 {
        ExitCode::SUCCESS
    } else {
        eprintln!("Erreurs : {errors}");
        ExitCode::FAILURE
    }
}

fn clean_directories(
    dir: &Path,
) -> Result<Vec<PathBuf>, io::Error> {
    let normalized_name = dir
        .file_name()
        .and_then(|value| value.to_str());

    match normalized_name {
        Some("errors") | Some("regression") => {
            Ok(vec![dir.to_path_buf()])
        }

        _ => {
            let mut directories = Vec::new();

            let errors_dir = dir.join("errors");
            let regression_dir = dir.join("regression");

            if errors_dir.is_dir() {
                directories.push(errors_dir);
            }

            if regression_dir.is_dir() {
                directories.push(regression_dir);
            }

            Ok(directories)
        }
    }
}

fn clean_expected_recursive(
    dir: &Path,
    removed: &mut usize,
    errors: &mut usize,
) -> Result<(), io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            clean_expected_recursive(
                &path,
                removed,
                errors,
            )?;
            continue;
        }

        if path.extension() == Some(OsStr::new("expected")) {
            match fs::remove_file(&path) {
                Ok(()) => {
                    println!(
                        "DELETE {}",
                        display_path(&path)
                    );

                    *removed += 1;
                }

                Err(error) => {
                    eprintln!(
                        "Impossible de supprimer {} : {}",
                        display_path(&path),
                        error
                    );

                    *errors += 1;
                }
            }
        }
    }

    Ok(())
}


// cargo run --bin test_runner -- --update
// cargo run --bin test_runner
// cargo run --bin test_runner -- test/regression
// cargo run --bin test_runner -- --clean