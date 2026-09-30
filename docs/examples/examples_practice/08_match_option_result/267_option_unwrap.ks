// ==================================================================
// Exemple 267 — unwrap et unwrap_or
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// unwrap() extrait la valeur (erreur si None) ; unwrap_or donne une valeur de secours.
// ------------------------------------------------------------------
// Sortie attendue :
//   42
//   0
//   42
// ==================================================================

let a = Some(42);
let b: Option<int> = None;
println(a.unwrap());
println(b.unwrap_or(0));
println(a.unwrap_or(0));
