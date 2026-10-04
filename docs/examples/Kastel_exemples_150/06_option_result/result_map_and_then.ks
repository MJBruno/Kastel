// ====================================================================
// Kastel — Chaîner des Result
// Notions : map, map_err, and_then
// Résultat attendu :
//   Ok(21)
//   Err(age négatif !)
//   Err(age négatif)
// ====================================================================

func verifier_age(age: int) -> Result<int, str> {
    if age < 0 {
        return Err("age négatif");
    }
    return Ok(age);
}

println(verifier_age(20).map(a => a + 1));
println(verifier_age(-1).map_err(m => m + " !"));
println(verifier_age(10).and_then(a => verifier_age(a - 20)));
