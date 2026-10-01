// ==================================================================
// Exemple 434 — Lire ligne par ligne
// Catégorie : Fichiers, JSON, système
// ------------------------------------------------------------------
// file_read_lines renvoie une liste de textes.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
//   A
//   B
//   C
// ==================================================================

file_write("lignes_kastel.txt", "a\nb\nc");
let lignes = file_read_lines("lignes_kastel.txt");
println(lignes.size());
for l in lignes {
    println(l.upper());
}
file_delete("lignes_kastel.txt");
