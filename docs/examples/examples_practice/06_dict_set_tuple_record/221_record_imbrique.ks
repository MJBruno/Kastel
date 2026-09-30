// ==================================================================
// Exemple 221 — Records imbriqués
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// Un champ peut contenir un autre record.
// ------------------------------------------------------------------
// Sortie attendue :
//   Paris
//   75001
// ==================================================================

let user = { nom: "Ada", adresse: { ville: "Paris", cp: 75001 } };
println(user.adresse.ville);
println(user.adresse.cp);
