// ==================================================================
// Exemple 043 — Extraire avec slice(debut, fin)
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// La fin est exclue ; les indices négatifs comptent depuis la fin.
// ------------------------------------------------------------------
// Sortie attendue :
//   Bon
//   jour
//   jour
// ==================================================================

let s = "Bonjour";
println(s.slice(0, 3));    // Bon
println(s.slice(3, 7));    // jour
println(s.slice(-4, 7));   // jour
