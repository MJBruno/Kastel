// ==================================================================
// Exemple 280 — and_then sur Result
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Enchaîner des étapes qui peuvent échouer.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ok(7)
//   Err(pas un nombre : abc)
// ==================================================================

func parser(s: str) -> Result<int, str> {
    if s.is_digit() {
        return Ok(s.to_int());
    }
    return Err("pas un nombre : " + s);
}

func racine_entiere(n: int) -> Result<int, str> {
    if n < 0 {
        return Err("négatif");
    }
    return Ok(floor(sqrt(float(n))));
}

println(parser("49").and_then(racine_entiere));
println(parser("abc").and_then(racine_entiere));
