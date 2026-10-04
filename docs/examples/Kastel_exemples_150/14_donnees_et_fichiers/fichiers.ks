// ====================================================================
// Kastel — Lire et écrire un fichier
// Notions : file_write, file_append, file_read_lines, file_exists, file_delete
// Résultat attendu :
//   true
//   3
//   ligne 1
//   false
// ====================================================================

let chemin = "kastel_demo.txt";

file_write(chemin, "ligne 1\nligne 2");
file_append(chemin, "\nligne 3");

println(file_exists(chemin));

let lignes = file_read_lines(chemin);
println(lignes.size());
println(lignes[0]);

file_delete(chemin);
println(file_exists(chemin));
