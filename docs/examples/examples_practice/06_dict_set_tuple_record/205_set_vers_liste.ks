// ==================================================================
// Exemple 205 — Ensemble vers liste
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// to_list() convertit ; utile pour dédoublonner une liste.
// ------------------------------------------------------------------
// Sortie attendue :
//   [3, 1, 2]
// ==================================================================

let uniques = Set(3, 1, 3, 2, 1).to_list();   // les doublons disparaissent
println(uniques);
