use std::process::ExitCode;

use kastel::app::application::Application;

/// Taille de la pile du thread d'interprétation.
///
/// Le lexer, le parser, le vérificateur de types, le compilateur et la VM
/// (rappels natifs imbriqués) sont récursifs. La pile du thread principal est
/// petite (1 Mo sous Windows, 8 Mo sous Linux) : on exécute donc tout dans un
/// thread dédié à pile large. Les limites de profondeur du langage
/// (`MAX_NESTING_DEPTH`, `MAX_EXPRESSION_DEPTH`, `MAX_NATIVE_DEPTH`) sont
/// dimensionnées pour tenir largement dans cette pile : elles produisent une
/// erreur Kastel plutôt qu'un débordement natif.
///
/// Tout le travail (et donc le registre du GC, propre au thread) reste dans ce
/// seul thread.
const INTERPRETER_STACK_SIZE: usize = 256 * 1024 * 1024;

fn main() -> ExitCode {
    let spawned = std::thread::Builder::new()
        .name("kastel".to_string())
        .stack_size(INTERPRETER_STACK_SIZE)
        .spawn(Application::run);

    match spawned {
        Ok(handle) => handle.join().unwrap_or(ExitCode::FAILURE),

        // Impossible de réserver la pile large : on tente quand même sur le
        // thread principal plutôt que de ne rien exécuter.
        Err(_) => Application::run(),
    }
}
