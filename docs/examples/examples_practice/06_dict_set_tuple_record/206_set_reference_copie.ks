// ==================================================================
// Exemple 206 — Ensemble : référence et copie
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// Comme pour les listes, copy() est nécessaire pour une copie indépendante.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

let a = Set(1, 2);
let b = a;             // même ensemble
let c = a.copy();      // copie
b.add(3);
c.add(99);
println(a.contains(3));    // true
println(a.contains(99));   // false
