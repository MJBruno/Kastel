// ==================================================================
// Exemple 443 — Écrire du JSON
// Catégorie : Fichiers, JSON, système
// ------------------------------------------------------------------
// json_encode transforme une valeur en texte JSON ; on peut le relire ensuite.
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   2
// ==================================================================

let original = {"a": 1, "b": [true, false]};
let texte = json_encode(original);
let relu = json_decode(texte);
println(relu["a"]);
println(relu["b"].size());
