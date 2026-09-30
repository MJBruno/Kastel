// ==================================================================
// Exemple 444 — Sauvegarder des données en JSON
// Catégorie : Fichiers, JSON, système
// ------------------------------------------------------------------
// Écrire un fichier JSON puis le relire.
// ------------------------------------------------------------------
// Sortie attendue :
//   sombre
//   14
// ==================================================================

let config = {"theme": "sombre", "taille": 14};
file_write("config_kastel.json", json_encode(config));

let relue = json_decode(file_read("config_kastel.json"));
println(relue["theme"]);
println(relue["taille"]);
file_delete("config_kastel.json");
