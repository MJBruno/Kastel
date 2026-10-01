// ==================================================================
// Exemple 439 — Statistiques d'un fichier
// Catégorie : Fichiers, JSON, système
// ------------------------------------------------------------------
// Lignes et mots d'un texte enregistré.
// ------------------------------------------------------------------
// Sortie attendue :
//   3 lignes, 6 mots
// ==================================================================

file_write("stats_kastel.txt", "un deux\ntrois quatre cinq\nsix");
let lignes = file_read_lines("stats_kastel.txt");
let mots = 0;
for l in lignes {
    mots += l.split(" ").size();
}
println("{} lignes, {} mots", lignes.size(), mots);
file_delete("stats_kastel.txt");
