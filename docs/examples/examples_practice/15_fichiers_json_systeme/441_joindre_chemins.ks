// ==================================================================
// Exemple 441 — Assembler un chemin
// Catégorie : Fichiers, JSON, système
// ------------------------------------------------------------------
// path_join accepte une liste de morceaux.
// ------------------------------------------------------------------
// Sortie attendue :
//   notes.txt
// ==================================================================

let p = path_join(["data", "2026", "notes.txt"]);
println(path_basename(p));
