// ==================================================================
// Exemple 189 — Dict vide, is_empty, clear
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// Créer, tester et vider.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
//   0
// ==================================================================

let d = dict();
println(d.is_empty());
d["k"] = 1;
println(d.is_empty());
d.clear();
println(d.size());
