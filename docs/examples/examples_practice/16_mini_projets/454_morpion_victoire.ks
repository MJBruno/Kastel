// ==================================================================
// Exemple 454 — Morpion : détecter une victoire
// Catégorie : Mini-projets
// ------------------------------------------------------------------
// Tester les 8 lignes gagnantes d'une grille de 9 cases.
// ------------------------------------------------------------------
// Sortie attendue :
//   X
//   personne
// ==================================================================

func gagnant(g: List<str>) -> str {
    let lignes = [
        [0, 1, 2], [3, 4, 5], [6, 7, 8],
        [0, 3, 6], [1, 4, 7], [2, 5, 8],
        [0, 4, 8], [2, 4, 6]
    ];
    for l in lignes {
        let a = g[l[0]];
        if a != "." && a == g[l[1]] && a == g[l[2]] {
            return a;
        }
    }
    return "personne";
}

let grille = ["X", "O", ".",
              ".", "X", "O",
              ".", ".", "X"];
println(gagnant(grille));
println(gagnant([".", ".", ".", ".", ".", ".", ".", ".", "."]));
