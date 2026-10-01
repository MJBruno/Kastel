// ==================================================================
// Exemple 442 — Lire du JSON
// Catégorie : Fichiers, JSON, système
// ------------------------------------------------------------------
// json_decode transforme un texte JSON en dict, liste, nombres...
// ------------------------------------------------------------------
// Sortie attendue :
//   Ada
//   37
//   2
// ==================================================================

let doc = json_decode("{\"nom\": \"Ada\", \"age\": 36, \"langages\": [\"kastel\", \"rust\"]}");
println(doc["nom"]);
println(doc["age"] + 1);
println(doc["langages"].size());
