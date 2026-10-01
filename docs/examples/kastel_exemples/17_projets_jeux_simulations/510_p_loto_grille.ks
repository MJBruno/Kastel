// ==================================================================
// Exemple 510 — Grille de loto : contrôle des gains
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : compter les numéros communs entre une grille et un tirage.
// ------------------------------------------------------------------
// Sortie attendue :
//   3 numéros gagnants
//   0 numéros gagnants
//   6 numéros gagnants
// ==================================================================

let tirage = Set(4, 8, 15, 16, 23, 42);
let grilles = [[4, 8, 15, 1, 2, 3], [1, 2, 3, 5, 6, 7], [4, 8, 15, 16, 23, 42]];

for g in grilles {
    let communs = Set(g[0], g[1], g[2], g[3], g[4], g[5]).intersection(tirage);
    println("{} numéros gagnants", communs.size());
}
