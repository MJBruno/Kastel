// ==================================================================
// Exemple 279 — map et map_err
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// map transforme le succès, map_err transforme l'erreur.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ok(5)
//   Err(erreur !)
// ==================================================================

let ok: Result<int, str> = Ok(4);
let ko: Result<int, str> = Err("erreur");
println(ok.map(x => x + 1));
println(ko.map_err(e => e + " !"));
