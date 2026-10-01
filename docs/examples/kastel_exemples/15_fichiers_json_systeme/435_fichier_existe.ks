// ==================================================================
// Exemple 435 — Tester l'existence
// Catégorie : Fichiers, JSON, système
// ------------------------------------------------------------------
// file_exists avant de lire évite une erreur.
// ------------------------------------------------------------------
// Sortie attendue :
//   false
//   true
//   false
// ==================================================================

println(file_exists("n_existe_pas_kastel.txt"));
file_write("existe_kastel.txt", "x");
println(file_exists("existe_kastel.txt"));
file_delete("existe_kastel.txt");
println(file_exists("existe_kastel.txt"));
