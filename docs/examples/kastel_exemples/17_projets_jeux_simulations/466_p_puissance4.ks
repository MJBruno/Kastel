// ==================================================================
// Exemple 466 — Puissance 4 : détecter quatre alignés
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : parcourir une grille 4 x 4 (simplifiée) et détecter 4 jetons identiques en ligne.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

func gagne(g: List<List<str>>, joueur: str) -> bool {
    let n = 4;
    for i in range(n) {
        let ligne = true;
        let colonne = true;
        for j in range(n) {
            if g[i][j] != joueur { ligne = false; }
            if g[j][i] != joueur { colonne = false; }
        }
        if ligne || colonne { return true; }
    }
    return false;
}

let g = [
    ["R", ".", ".", "."],
    ["R", "J", ".", "."],
    ["R", "J", ".", "."],
    ["R", "J", ".", "."]
];
println(gagne(g, "R"));
println(gagne(g, "J"));
