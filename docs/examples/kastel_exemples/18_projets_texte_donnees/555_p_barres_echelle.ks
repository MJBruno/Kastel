// ==================================================================
// Exemple 555 — Barres mises à l'échelle
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : ramener des valeurs quelconques à une largeur maximale de 8.
// ------------------------------------------------------------------
// Sortie attendue :
//   a ####
//   b ########
//   c ##
// ==================================================================

let valeurs = [("a", 10), ("b", 20), ("c", 5)];
let maximum = 0;
for v in valeurs { maximum = max(maximum, v[1]); }

for v in valeurs {
    let largeur = idiv(v[1] * 8, maximum);
    println("{} {}", v[0], "#".repeat(largeur));
}
