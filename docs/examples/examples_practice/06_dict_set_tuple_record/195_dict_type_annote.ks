// ==================================================================
// Exemple 195 — Annoter un dict
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// Dict<str, int> précise le type des clés et des valeurs.
// ------------------------------------------------------------------
// Sortie attendue :
//   10
// ==================================================================

let stock: Dict<str, int> = {"pommes": 12, "poires": 7};
stock["pommes"] = stock["pommes"] - 2;
println(stock["pommes"]);
