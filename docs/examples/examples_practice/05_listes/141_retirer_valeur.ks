// ==================================================================
// Exemple 141 — Retirer une valeur avec remove()
// Catégorie : Listes
// ------------------------------------------------------------------
// remove(x) supprime la première occurrence et renvoie true si elle existait.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
//   [5, 7, 6]
// ==================================================================

let v = [5, 6, 7, 6];
println(v.remove(6));   // true
println(v.remove(42));  // false
println(v);
