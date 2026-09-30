// ==================================================================
// Exemple 045 — replace et replace_all
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// replace ne change que la première occurrence, replace_all toutes.
// ------------------------------------------------------------------
// Sortie attendue :
//   le chat, un chien
//   le chat, le chien
// ==================================================================

let s = "un chat, un chien";
println(s.replace("un", "le"));
println(s.replace_all("un", "le"));
