// ==================================================================
// Exemple 286 — ? et finally ensemble
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Le bloc finally s'exécute même si ? termine la fonction plus tôt.
// ------------------------------------------------------------------
// Sortie attendue :
//   Err(boom)
//   1
// ==================================================================

let nettoyages = 0;

func echoue() -> Result<int, str> {
    return Err("boom");
}

func executer() -> Result<int, str> {
    try {
        let v = echoue()?;
        return Ok(v);
    } finally {
        nettoyages += 1;
    }
}

println(executer());
println(nettoyages);
