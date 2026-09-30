// ==================================================================
// Exemple 283 — ok() et err()
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Convertir un Result en Option.
// ------------------------------------------------------------------
// Sortie attendue :
//   Some(1)
//   None
//   Some(x)
// ==================================================================

let a: Result<int, str> = Ok(1);
let b: Result<int, str> = Err("x");
println(a.ok());
println(b.ok());
println(b.err());
