// ==================================================================
// Exemple 432 — Écrire puis lire un fichier
// Catégorie : Fichiers, JSON, système
// ------------------------------------------------------------------
// file_write crée (ou remplace) le fichier ; file_read renvoie tout son contenu.
// ------------------------------------------------------------------
// Sortie attendue :
//   Bonjour fichier !
// ==================================================================

file_write("demo_kastel.txt", "Bonjour fichier !");
println(file_read("demo_kastel.txt"));
file_delete("demo_kastel.txt");
