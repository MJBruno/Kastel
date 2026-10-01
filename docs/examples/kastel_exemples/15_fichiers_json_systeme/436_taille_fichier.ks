// ==================================================================
// Exemple 436 — Taille d'un fichier
// Catégorie : Fichiers, JSON, système
// ------------------------------------------------------------------
// file_size donne le nombre d'octets.
// ------------------------------------------------------------------
// Sortie attendue :
//   5
// ==================================================================

file_write("taille_kastel.txt", "12345");
println(file_size("taille_kastel.txt"));
file_delete("taille_kastel.txt");
