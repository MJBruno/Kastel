// ==================================================================
// Exemple 277 — Diviser sans planter
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Renvoyer une erreur plutôt que lever une exception.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ok(2.5)
//   Err(division par zéro)
// ==================================================================

func diviser(a: int, b: int) -> Result<float, str> {
    if b == 0 {
        return Err("division par zéro");
    }
    return Ok(a / b);
}

println(diviser(10, 4));
println(diviser(1, 0));
