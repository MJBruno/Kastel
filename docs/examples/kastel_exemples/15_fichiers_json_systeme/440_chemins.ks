// ==================================================================
// Exemple 440 — Manipuler des chemins
// Catégorie : Fichiers, JSON, système
// ------------------------------------------------------------------
// path_basename, path_dirname, path_extension, path_stem.
// ------------------------------------------------------------------
// Sortie attendue :
//   rapport.final.txt
//   txt
//   rapport.final
//   dossier/sous
// ==================================================================

let chemin = "dossier/sous/rapport.final.txt";
println(path_basename(chemin));
println(path_extension(chemin));
println(path_stem(chemin));
println(path_dirname(chemin));
