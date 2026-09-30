// ==================================================================
// Exemple 266 — Option : une valeur peut manquer
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Some(x) contient une valeur, None représente l'absence.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   true
//   Some(5)
//   None
// ==================================================================

let a: Option<int> = Some(5);
let b: Option<int> = None;
println(a.is_some());
println(b.is_none());
println(a);
println(b);
