use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

#[derive(Debug)]
struct TestResult {
    path: PathBuf,
    passed: bool,
    message: String,
}

fn main() -> ExitCode {
    let args = env::args().skip(1);

    let mut update = false;
    let mut clean = false;
    let mut test_dir = PathBuf::from("test");

    for arg in args {
        match arg.as_str() {
            "--update" => {
                update = true;
            }

            "--clean" => {
                clean = true;
            }

            "--help" | "-h" => {
                print_help();
                return ExitCode::SUCCESS;
            }

            _ => {
                test_dir = PathBuf::from(arg);
            }
        }
    }

    if update && clean {
        eprintln!("Erreur : --update et --clean ne peuvent pas être utilisés ensemble.");

        return ExitCode::FAILURE;
    }

    if !test_dir.is_dir() {
        eprintln!(
            "Erreur : dossier de tests introuvable : {}",
            test_dir.display()
        );

        return ExitCode::FAILURE;
    }

    /*
     * Mode nettoyage.
     */
    if clean {
        return clean_expected_files(&test_dir);
    }

    println!("Kastel Test Runner");
    println!("==================");
    println!("Dossier : {}", test_dir.display());

    if update {
        println!("Mode : UPDATE");
    } else {
        println!("Mode : CHECK");
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
     * Recherche récursive des fichiers .ks.
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
        println!("Aucun test Kastel trouvé.");
        return ExitCode::SUCCESS;
    }

    let mut results = Vec::with_capacity(tests.len());

    for test in tests {
        let result = run_test(&executable, &test, update);

        if result.passed {
            println!("PASS {}", display_path(&result.path));
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

fn print_help() {
    println!("Kastel Test Runner");
    println!();
    println!("Usage:");
    println!("  cargo run --bin test_runner");
    println!("  cargo run --bin test_runner -- --update");
    println!("  cargo run --bin test_runner -- --clean");
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
        .map_err(|error| format!("Impossible de lancer Cargo : {error}"))?;

    if !status.success() {
        return Err("Erreur : impossible de compiler Kastel.".to_string());
    }

    let current_exe = env::current_exe()
        .map_err(|error| format!("Impossible de déterminer le chemin du test runner : {error}"))?;

    let debug_dir = current_exe
        .parent()
        .ok_or_else(|| "Répertoire de l'exécutable introuvable.".to_string())?;

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
    if cfg!(windows) { "cargo.exe" } else { "cargo" }
}

fn collect_tests(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut tests = Vec::new();

    collect_tests_recursive(dir, &mut tests)?;

    tests.sort();

    Ok(tests)
}

fn collect_tests_recursive(dir: &Path, tests: &mut Vec<PathBuf>) -> Result<(), io::Error> {
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

fn run_test(executable: &Path, test_path: &Path, update: bool) -> TestResult {
    let output = match Command::new(executable).arg(test_path).output() {
        Ok(output) => output,

        Err(error) => {
            return TestResult {
                path: test_path.to_path_buf(),
                passed: false,
                message: format!("Impossible d'exécuter Kastel : {error}"),
            };
        }
    };

    let stdout = normalize_output(&String::from_utf8_lossy(&output.stdout));

    let stderr = normalize_output(&String::from_utf8_lossy(&output.stderr));

    let is_error_test = is_error_test(test_path);

    let has_kastel_error = stdout.contains("Erreur dans")
        || stderr.contains("Erreur dans")
        || stdout.contains("Erreur de compilation")
        || stderr.contains("Erreur de compilation")
        || stdout.contains("Erreur d'exécution")
        || stderr.contains("Erreur d'exécution")
        || stdout.contains("Erreur(s) de parsing")
        || stderr.contains("Erreur(s) de parsing");

    /*
     * Les tests situés dans test/errors/
     * doivent produire une erreur Kastel.
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
            message: "Une erreur Kastel était attendue, mais aucune erreur n'a été détectée."
                .to_string(),
        };
    }

    /*
     * Pour les tests normaux, toute erreur Kastel
     * fait immédiatement échouer le test.
     */
    if has_kastel_error {
        return TestResult {
            path: test_path.to_path_buf(),
            passed: false,
            message: format!(
                "Erreur Kastel inattendue.\n\nstdout:\n{}\n\nstderr:\n{}",
                stdout, stderr
            ),
        };
    }

    /*
     * Le fichier attendu possède le même nom que le .ks
     * avec l'extension .expected.
     *
     * Exemple :
     *   nested_calls.ks
     *   nested_calls.expected
     */
    let expected_path = test_path.with_extension("expected");

    /*
     * Mode UPDATE :
     * créer ou remplacer le fichier attendu.
     */
    if update {
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
            message: format!("Sortie attendue générée : {}", expected_path.display()),
        };
    }

    /*
     * Mode CHECK :
     * le fichier .expected doit exister.
     */
    if !expected_path.is_file() {
        return TestResult {
            path: test_path.to_path_buf(),
            passed: false,
            message: format!("Fichier attendu manquant : {}", expected_path.display()),
        };
    }

    let expected = match fs::read_to_string(&expected_path) {
        Ok(content) => normalize_output(&content),

        Err(error) => {
            return TestResult {
                path: test_path.to_path_buf(),
                passed: false,
                message: format!("Impossible de lire {} : {}", expected_path.display(), error),
            };
        }
    };

    /*
     * Comparaison exacte de la sortie.
     */
    if stdout != expected {
        return TestResult {
            path: test_path.to_path_buf(),
            passed: false,
            message: format!(
                "Sortie différente de la sortie attendue.\n\n\
                 Attendu:\n{}\n\n\
                 Obtenu:\n{}",
                expected, stdout
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
    path.components()
        .any(|component| component.as_os_str() == OsStr::new("errors"))
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
    let passed = results.iter().filter(|result| result.passed).count();

    let failed = results.len() - passed;

    println!();
    println!("==================");
    println!("Total  : {}", results.len());
    println!("Passed : {passed}");
    println!("Failed : {failed}");
    println!("==================");
}

/*
 * Supprime récursivement tous les fichiers .expected.
 */
fn clean_expected_files(dir: &Path) -> ExitCode {
    let mut removed = 0usize;
    let mut errors = 0usize;

    println!("Kastel Test Runner");
    println!("==================");
    println!("Mode : CLEAN");
    println!("Dossier : {}", dir.display());
    println!();

    match clean_expected_recursive(dir, &mut removed, &mut errors) {
        Ok(()) => {
            println!();
            println!("Fichiers .expected supprimés : {removed}");

            if errors == 0 {
                ExitCode::SUCCESS
            } else {
                eprintln!("Erreurs : {errors}");
                ExitCode::FAILURE
            }
        }

        Err(error) => {
            eprintln!("Erreur pendant le nettoyage : {error}");

            ExitCode::FAILURE
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
            clean_expected_recursive(&path, removed, errors)?;
            continue;
        }

        if path.extension() == Some(OsStr::new("expected")) {
            match fs::remove_file(&path) {
                Ok(()) => {
                    println!("DELETE {}", display_path(&path));

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
