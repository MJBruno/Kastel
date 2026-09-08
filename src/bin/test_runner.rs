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
    let args: Vec<String> = env::args().collect();
    let test_dir = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("test"));
    if !test_dir.is_dir() {
        eprintln!(
            "Erreur : dossier de tests introuvable : {}",
            test_dir.display()
        );
        return ExitCode::FAILURE;
    }
    println!("Kastel Test Runner");
    println!("==================");
    println!("Dossier : {}", test_dir.display());
    println!();
    if let Err(error) = build_kastel() {
        eprintln!("Erreur de compilation de Kastel : {error}");
        return ExitCode::FAILURE;
    }
    let executable = match kastel_executable() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Erreur : {error}");
            return ExitCode::FAILURE;
        }
    };
    let all_tests = match collect_all_kastel_files(&test_dir) {
        Ok(count) => count,
        Err(error) => {
            eprintln!("Erreur lecture des fichiers de test : {error}");
            return ExitCode::FAILURE;
        }
    };
    let tests = match collect_tests(&test_dir) {
        Ok(tests) => tests,
        Err(error) => {
            eprintln!("Erreur lecture des tests automatisés : {error}");
            return ExitCode::FAILURE;
        }
    };
    let skipped = all_tests.saturating_sub(tests.len());
    println!("Tests Kastel : {all_tests}");
    println!("Tests automatisés : {}", tests.len());
    println!("Tests ignorés : {skipped}");
    println!();
    if tests.is_empty() {
        println!("Aucun test automatisé trouvé.");
        return ExitCode::SUCCESS;
    }
    let mut results = Vec::with_capacity(tests.len());
    for test in tests {
        let result = run_test(&executable, &test);
        if result.passed {
            println!("PASS {}", display_path(&result.path));
        } else {
            println!("FAIL {}", display_path(&result.path));
            if !result.message.is_empty() {
                println!("{}", indent(&result.message));
            }
            println!();
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
fn build_kastel() -> Result<(), String> {
    let status = Command::new(cargo_program())
        .args(["build", "--quiet", "--bin", "kastel"])
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`cargo build --bin kastel` a échoué : {status}"))
    }
}
fn cargo_program() -> &'static str {
    if cfg!(windows) { "cargo.exe" } else { "cargo" }
}
fn kastel_executable() -> Result<PathBuf, String> {
    let current = env::current_exe().map_err(|error| error.to_string())?;
    let debug_dir = current
        .parent()
        .ok_or_else(|| "répertoire de l'exécutable introuvable".to_string())?;
    let executable_name = if cfg!(windows) {
        "kastel.exe"
    } else {
        "kastel"
    };
    let executable = debug_dir.join(executable_name);
    if !executable.is_file() {
        return Err(format!(
            "exécutable Kastel introuvable : {}",
            executable.display()
        ));
    }
    Ok(executable)
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
        if path.extension() != Some(OsStr::new("ks")) {
            continue;
        }
        let expected = path.with_extension("expected");
        let stderr = path.with_extension("stderr");
        if expected.is_file() || stderr.is_file() {
            tests.push(path);
        }
    }
    Ok(())
}
fn collect_all_kastel_files(dir: &Path) -> Result<usize, io::Error> {
    let mut count = 0;
    collect_all_kastel_files_recursive(dir, &mut count)?;
    Ok(count)
}
fn collect_all_kastel_files_recursive(dir: &Path, count: &mut usize) -> Result<(), io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_all_kastel_files_recursive(&path, count)?;
            continue;
        }
        if path.extension() == Some(OsStr::new("ks")) {
            *count += 1;
        }
    }
    Ok(())
}
fn run_test(executable: &Path, test_path: &Path) -> TestResult {
    let expected_path = test_path.with_extension("expected");
    let stderr_path = test_path.with_extension("stderr");
    let output = match Command::new(executable).arg(test_path).output() {
        Ok(output) => output,
        Err(error) => {
            return TestResult {
                path: test_path.to_path_buf(),
                passed: false,
                message: format!("impossible d'exécuter Kastel : {error}"),
            };
        }
    };
    let stdout = normalize_output(&String::from_utf8_lossy(&output.stdout));
    let stderr = normalize_output(&String::from_utf8_lossy(&output.stderr));
    let expected_stdout = if expected_path.is_file() {
        match fs::read_to_string(&expected_path) {
            Ok(value) => Some(normalize_output(&value)),
            Err(error) => {
                return TestResult {
                    path: test_path.to_path_buf(),
                    passed: false,
                    message: format!("impossible de lire {} : {error}", expected_path.display()),
                };
            }
        }
    } else {
        None
    };
    let expected_stderr = if stderr_path.is_file() {
        match fs::read_to_string(&stderr_path) {
            Ok(value) => Some(normalize_output(&value)),
            Err(error) => {
                return TestResult {
                    path: test_path.to_path_buf(),
                    passed: false,
                    message: format!("impossible de lire {} : {error}", stderr_path.display()),
                };
            }
        }
    } else {
        None
    }; /* * Cas normal : * * .expected présent * → le programme doit réussir * → stdout doit correspondre * → stderr doit être vide */
    if let Some(expected) = expected_stdout {
        if !output.status.success() {
            return TestResult {
                path: test_path.to_path_buf(),
                passed: false,
                message: format!("Kastel a échoué.\n\nstderr:\n{}", stderr),
            };
        }
        if stdout != expected {
            return TestResult {
                path: test_path.to_path_buf(),
                passed: false,
                message: format!(
                    "stdout différent\n\nattendu:\n{}\n\nobtenu:\n{}",
                    expected, stdout
                ),
            };
        }
        if !stderr.is_empty() {
            return TestResult {
                path: test_path.to_path_buf(),
                passed: false,
                message: format!("stderr inattendu:\n{}", stderr),
            };
        }
        return TestResult {
            path: test_path.to_path_buf(),
            passed: true,
            message: String::new(),
        };
    } /* * Cas erreur : * * .stderr présent * → le programme doit échouer * → stderr doit correspondre */
    if let Some(expected) = expected_stderr {
        if output.status.success() {
            return TestResult {
                path: test_path.to_path_buf(),
                passed: false,
                message: "Kastel a réussi alors qu'une erreur était attendue.".to_string(),
            };
        }
        if stderr != expected {
            return TestResult {
                path: test_path.to_path_buf(),
                passed: false,
                message: format!(
                    "stderr différent\n\nattendu:\n{}\n\nobtenu:\n{}",
                    expected, stderr
                ),
            };
        }
        return TestResult {
            path: test_path.to_path_buf(),
            passed: true,
            message: String::new(),
        };
    }
    TestResult {
        path: test_path.to_path_buf(),
        passed: false,
        message: "aucun fichier .expected ou .stderr".to_string(),
    }
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
        .map(|line| format!(" {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}
fn print_summary(results: &[TestResult]) {
    let passed = results.iter().filter(|result| result.passed).count();
    let failed = results.len() - passed;
    println!();
    println!("==================");
    println!("Total : {}", results.len());
    println!("Passed : {passed}");
    println!("Failed : {failed}");
    println!("==================");
}
