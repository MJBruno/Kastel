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

fn main() -> ExitCode {
    let test_dir = env::args()
        .nth(1)
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

    let executable = match build_kastel() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

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
        let result = run_test(&executable, &test);

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

    let current = env::current_exe()
        .map_err(|error| error.to_string())?;

    let debug_dir = current
        .parent()
        .ok_or_else(|| {
            "Répertoire de l'exécutable introuvable".to_string()
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

fn collect_tests(
    dir: &Path,
) -> Result<Vec<PathBuf>, io::Error> {
    let mut tests = Vec::new();

    collect_tests_recursive(dir, &mut tests)?;

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

    /*
     * IMPORTANT :
     *
     * Le binaire Kastel actuel retourne toujours SUCCESS
     * depuis src/main.rs, même lorsque Application::run()
     * retourne FAILURE.
     *
     * On ne peut donc pas utiliser uniquement output.status.success().
     *
     * On détecte explicitement le diagnostic Kastel.
     */
    let has_kastel_error =
        stdout.contains("Erreur dans")
            || stderr.contains("Erreur dans")
            || stdout.contains("Erreur de compilation")
            || stderr.contains("Erreur de compilation")
            || stdout.contains("Erreur d'exécution")
            || stderr.contains("Erreur d'exécution")
            || stdout.contains("Erreur(s) de parsing")
            || stderr.contains("Erreur(s) de parsing");

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
    let passed =
        results.iter().filter(|result| result.passed).count();

    let failed = results.len() - passed;

    println!();
    println!("==================");
    println!("Total  : {}", results.len());
    println!("Passed : {passed}");
    println!("Failed : {failed}");
    println!("==================");
}