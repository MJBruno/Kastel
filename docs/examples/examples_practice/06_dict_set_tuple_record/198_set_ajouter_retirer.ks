// ==================================================================
// Exemple 198 — add, remove, contains
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// Manipuler un ensemble.
// ------------------------------------------------------------------
// Sortie attendue :
//   {2, 3}
//   true
//   false
// ==================================================================

let s = Set(1, 2);
s.add(3);
s.add(3);          // sans effet : déjà présent
s.remove(1);
println(s);
println(s.contains(2));
println(s.contains(1));
