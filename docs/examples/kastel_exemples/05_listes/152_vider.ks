// ==================================================================
// Exemple 152 — Vider et tester si vide
// Catégorie : Listes
// ------------------------------------------------------------------
// clear() supprime tout ; is_empty() teste.
// ------------------------------------------------------------------
// Sortie attendue :
//   false
//   true
//   0
// ==================================================================

let v = [1, 2, 3];
println(v.is_empty());
v.clear();
println(v.is_empty());
println(v.size());
