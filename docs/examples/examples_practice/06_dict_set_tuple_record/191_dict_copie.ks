// ==================================================================
// Exemple 191 — Copier un dict
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// Comme les listes, les dicts sont partagés : copy() donne une copie indépendante.
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   999
// ==================================================================

let a = {"x": 1};
let b = a.copy();
b["x"] = 999;
println(a["x"]);
println(b["x"]);
