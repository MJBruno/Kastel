// ==================================================================
// Exemple 211 — size, contains, index_of
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// Un tuple offre les mêmes lectures qu'une liste.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
//   true
//   2
//   10
//   30
// ==================================================================

let t = (10, 20, 30);
println(t.size());
println(t.contains(20));
println(t.index_of(30));
println(t.first());
println(t.last());
