// ==================================================================
// Exemple 042 — Chercher la position d'un texte
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// index_of renvoie la première position, ou -1 si absent ; last_index_of la dernière.
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   3
//   -1
// ==================================================================

let s = "banane";
println(s.index_of("an"));        // 1
println(s.last_index_of("an"));   // 3
println(s.index_of("z"));         // -1
