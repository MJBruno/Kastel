// ====================================================================
// Kastel — L'opérateur ? sur Result
// Notions : propagation d'erreur
// Résultat attendu :
//   Ok(42)
//   Err(pas un nombre : x)
// ====================================================================

func lire_nombre(texte: str) -> Result<int, str> {
    if !texte.is_digit() {
        return Err("pas un nombre : " + texte);
    }
    return Ok(texte.to_int());
}

func additionner(a: str, b: str) -> Result<int, str> {
    let x = lire_nombre(a)?;
    let y = lire_nombre(b)?;
    return Ok(x + y);
}

println(additionner("20", "22"));
println(additionner("20", "x"));
