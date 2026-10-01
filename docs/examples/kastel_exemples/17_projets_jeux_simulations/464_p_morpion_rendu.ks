// ==================================================================
// Exemple 464 — Morpion : afficher la grille
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : dessiner une grille de 3 x 3 à partir d'une liste de 9 cases.
// ------------------------------------------------------------------
// Sortie attendue :
//   X | O | X
//   . | O | .
//   . | O | X
// ==================================================================

let grille = ["X", "O", "X",
              ".", "O", ".",
              ".", "O", "X"];

func rendre(g: List<str>) {
    for ligne in range(3) {
        let cases = [g[3 * ligne], g[3 * ligne + 1], g[3 * ligne + 2]];
        println(" | ".join(cases));
    }
}

rendre(grille);
