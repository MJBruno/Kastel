// ==================================================================
// Exemple 222 — Record ou dict ?
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// Record : forme fixe, clés-identifiants. Dict : clés textuelles, ajout et retrait libres.
// ------------------------------------------------------------------
// Sortie attendue :
//   Bruno
//   Bruno
//   2
// ==================================================================

let record = { nom: "Bruno" };
let dico = {"nom": "Bruno"};
println(record.nom);      // accès par point
println(dico["nom"]);     // accès par crochets
dico["age"] = 25;         // un dict peut grandir
println(dico.size());
