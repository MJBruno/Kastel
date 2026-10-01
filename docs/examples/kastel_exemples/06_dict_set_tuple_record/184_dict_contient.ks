// ==================================================================
// Exemple 184 — Tester l'existence d'une clé
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// contains(cle) teste les CLÉS, pas les valeurs.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

let d = {"a": 1, "b": 2};
println(d.contains("a"));
println(d.contains("z"));
