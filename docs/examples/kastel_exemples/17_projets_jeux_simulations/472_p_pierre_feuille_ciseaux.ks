// ==================================================================
// Exemple 472 — Pierre, feuille, ciseaux
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : arbitrer une série de manches.
// ------------------------------------------------------------------
// Sortie attendue :
//   1 - 2
// ==================================================================

func vainqueur(a: str, b: str) -> int {
    if a == b { return 0; }
    if (a == "pierre" && b == "ciseaux") || (a == "feuille" && b == "pierre") || (a == "ciseaux" && b == "feuille") {
        return 1;
    }
    return 2;
}

let j1 = ["pierre", "feuille", "ciseaux", "pierre"];
let j2 = ["ciseaux", "feuille", "pierre", "feuille"];
let score1 = 0;
let score2 = 0;
for i in range(j1.size()) {
    let v = vainqueur(j1[i], j2[i]);
    if v == 1 { score1 += 1; }
    if v == 2 { score2 += 1; }
}
println("{} - {}", score1, score2);
