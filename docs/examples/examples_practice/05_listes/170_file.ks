// ==================================================================
// Exemple 170 — Une file (FIFO)
// Catégorie : Listes
// ------------------------------------------------------------------
// add pour enfiler, remove_at(0) pour défiler.
// ------------------------------------------------------------------
// Sortie attendue :
//   premier
//   deuxieme
//   1
// ==================================================================

let file = [];
file.add("premier");
file.add("deuxieme");
file.add("troisieme");
println(file.remove_at(0));
println(file.remove_at(0));
println(file.size());
