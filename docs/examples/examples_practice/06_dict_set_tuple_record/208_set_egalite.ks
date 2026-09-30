// ==================================================================
// Exemple 208 — Comparer deux ensembles
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// equals() ignore l'ordre d'insertion.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

let a = Set(1, 2, 3);
let b = Set(3, 2, 1);
println(a.equals(b));
println(a.equals(Set(1, 2)));
