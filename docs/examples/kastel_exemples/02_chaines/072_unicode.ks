// ==================================================================
// Exemple 072 — Texte accentué et Unicode
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// size() et reverse() raisonnent en caractères, pas en octets.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
//   été
//   ÉTÉ
// ==================================================================

let s = "été";
println(s.size());       // 3
println(s.reverse());    // été
println(s.upper());      // ÉTÉ
