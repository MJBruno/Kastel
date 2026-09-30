// ==================================================================
// Exemple 185 — Valeur par défaut avec get_or
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// get_or(cle, defaut) évite une erreur quand la clé manque.
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   0
// ==================================================================

let d = {"a": 1};
println(d.get_or("a", 0));
println(d.get_or("z", 0));
