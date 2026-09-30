// ==================================================================
// Exemple 038 — Longueur d'une chaîne
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// size() compte les caractères Unicode, pas les octets.
// ------------------------------------------------------------------
// Sortie attendue :
//   6
//   4
//   0
// ==================================================================

println("Kastel".size());   // 6
println("café".size());     // 4 (le é compte pour 1)
println("".size());         // 0
