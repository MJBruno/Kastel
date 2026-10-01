// ==================================================================
// Exemple 433 — Ajouter à la fin d'un fichier
// Catégorie : Fichiers, JSON, système
// ------------------------------------------------------------------
// file_append n'efface pas le contenu existant.
// ------------------------------------------------------------------
// Sortie attendue :
//   ligne 1
//   ligne 2
//   (ligne vide)
// ==================================================================

file_write("journal_kastel.txt", "ligne 1\n");
file_append("journal_kastel.txt", "ligne 2\n");
println(file_read("journal_kastel.txt"));
file_delete("journal_kastel.txt");
