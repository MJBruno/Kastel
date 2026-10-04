// ====================================================================
// Kastel — Convertir Option en Result
// Notions : ok_or
// Résultat attendu :
//   Ok(3)
//   Err(vide)
// ====================================================================

let plein: Option<int> = Some(3);
let vide: Option<int> = None;

println(plein.ok_or("vide"));
println(vide.ok_or("vide"));
