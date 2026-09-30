// ==================================================================
// Exemple 276 — Result : réussite ou erreur
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Ok(valeur) pour un succès, Err(raison) pour un échec.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   true
//   Ok(10)
//   Err(échec)
// ==================================================================

let bon: Result<int, str> = Ok(10);
let mauvais: Result<int, str> = Err("échec");
println(bon.is_ok());
println(mauvais.is_err());
println(bon);
println(mauvais);
