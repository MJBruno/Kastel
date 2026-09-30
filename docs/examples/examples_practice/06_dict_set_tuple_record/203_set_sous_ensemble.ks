// ==================================================================
// Exemple 203 — Sous-ensemble et sur-ensemble
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// is_subset et is_superset.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   true
//   false
// ==================================================================

let petit = Set(1, 2);
let grand = Set(1, 2, 3);
println(petit.is_subset(grand));
println(grand.is_superset(petit));
println(grand.is_subset(petit));
