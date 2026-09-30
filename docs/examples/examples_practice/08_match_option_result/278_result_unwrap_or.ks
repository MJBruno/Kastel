// ==================================================================
// Exemple 278 — unwrap, unwrap_err, unwrap_or
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Extraire la valeur de succès, l'erreur, ou une valeur de secours.
// ------------------------------------------------------------------
// Sortie attendue :
//   7
//   oups
//   0
// ==================================================================

let ok: Result<int, str> = Ok(7);
let ko: Result<int, str> = Err("oups");
println(ok.unwrap());
println(ko.unwrap_err());
println(ko.unwrap_or(0));
