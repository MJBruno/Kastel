// ====================================================================
// Kastel — Manipuler des chemins
// Notions : path_join, path_basename, path_extension, path_stem, path_dirname
// Résultat attendu :
//   rapport.pdf
//   pdf
//   rapport
// ====================================================================

let chemin = path_join(["documents", "rapport.pdf"]);

println(path_basename(chemin));
println(path_extension(chemin));
println(path_stem(chemin));
