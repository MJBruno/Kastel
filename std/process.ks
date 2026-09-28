// std/process.ks
//
// Lance un programme externe et récupère stdout/stderr/code, au-dessus
// de la native process_run() (voir src/stdlib/process.rs —
// std::process::Command, appel système impossible en Kastel pur).
//
// Volontairement SANS shell : `run("ls", ["-la", dossier])` exécute
// directement le binaire `ls` avec ces deux arguments, jamais
// `/bin/sh -c "ls -la " + dossier` — donc pas d'injection de commande
// même si `dossier` contient des caractères spéciaux du shell.
//
// Usage :
//
//   import std.process;
//
//   match process.run("git", ["status", "--short"]) {
//       Ok(output) => { println(output.stdout()); }
//       Err(message) => { println("echec: " + message); }
//   }

export class ProcessOutput {
    private let stdout_text: str = "";
    private let stderr_text: str = "";
    private let exit_code: int = 0;
    private let ok: bool = false;

    private func initialize(stdout_text: str, stderr_text: str, exit_code: int, ok: bool) {
        self.stdout_text = stdout_text;
        self.stderr_text = stderr_text;
        self.exit_code = exit_code;
        self.ok = ok;
    }

    // `raw` est le Record natif { stdout, stderr, code, success }
    // renvoyé par process_run().
    static func from_native(raw) -> ProcessOutput {
        return new ProcessOutput(raw.stdout, raw.stderr, raw.code, raw.success);
    }

    func stdout() -> str {
        return self.stdout_text;
    }

    func stderr() -> str {
        return self.stderr_text;
    }

    // Code de sortie du processus, ou -1 s'il a été terminé par un
    // signal (pas de vrai code de sortie dans ce cas).
    func code() -> int {
        return self.exit_code;
    }

    func success() -> bool {
        return self.ok;
    }

    func to_string() -> str {
        return "ProcessOutput(code=" + str(self.exit_code) + ", success=" + str(self.ok) + ")";
    }
}

export func run(program: str, arguments: List<str>) -> Result<ProcessOutput, str> {
    try {
        let raw = process_run(program, arguments);
        return Ok(ProcessOutput.from_native(raw));
    } catch (error) {
        return Err(error);
    }
}

// Raccourci pour un programme sans arguments.
export func run0(program: str) -> Result<ProcessOutput, str> {
    return run(program, []);
}

// Comme run(), mais renvoie aussi Err si le programme s'est terminé
// avec un code de sortie non nul (le message d'erreur reprend
// stderr) — pratique quand seul le succès du programme importe, sans
// avoir à vérifier .success() soi-même.
export func run_checked(program: str, arguments: List<str>) -> Result<ProcessOutput, str> {
    match run(program, arguments) {
        Ok(output) => {
            if output.success() {
                return Ok(output);
            }
            return Err(
                program + " a quitte avec le code " + str(output.code())
                    + " : " + output.stderr()
            );
        }
        Err(error) => {
            return Err(error);
        }
    }
}
