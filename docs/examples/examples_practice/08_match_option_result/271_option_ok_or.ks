// ==================================================================
// Exemple 271 — ok_or : Option vers Result
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Donner une erreur explicite à l'absence de valeur.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ok(3)
//   Err(absent)
// ==================================================================

let a = Some(3);
let b: Option<int> = None;
println(a.ok_or("absent"));
println(b.ok_or("absent"));
