// ==================================================================
// Exemple 282 — L'opérateur ? sur Result
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// En cas d'Err, la fonction se termine et renvoie cette même erreur.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ok(42)
//   Err(invalide : zz)
// ==================================================================

func parser(s: str) -> Result<int, str> {
    if s.is_digit() {
        return Ok(s.to_int());
    }
    return Err("invalide : " + s);
}

func additionner(a: str, b: str) -> Result<int, str> {
    let x = parser(a)?;
    let y = parser(b)?;
    return Ok(x + y);
}

println(additionner("20", "22"));
println(additionner("20", "zz"));
