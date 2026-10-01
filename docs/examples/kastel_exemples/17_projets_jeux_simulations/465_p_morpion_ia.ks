// ==================================================================
// Exemple 465 — Morpion : coup de l'ordinateur
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : l'ordinateur gagne s'il le peut, sinon bloque l'adversaire, sinon joue le centre puis la première case libre.
// ------------------------------------------------------------------
// Sortie attendue :
//   2
//   2
//   4
// ==================================================================

let lignes = [[0,1,2],[3,4,5],[6,7,8],[0,3,6],[1,4,7],[2,5,8],[0,4,8],[2,4,6]];

// Cherche une case qui complète une ligne pour le joueur donné.
func coup_gagnant(g: List<str>, joueur: str) -> int {
    for l in lignes {
        let n = 0;
        let vide = -1;
        for i in l {
            if g[i] == joueur { n += 1; }
            if g[i] == "." { vide = i; }
        }
        if n == 2 && vide != -1 {
            return vide;
        }
    }
    return -1;
}

func choisir(g: List<str>) -> int {
    let c = coup_gagnant(g, "O");     // gagner
    if c != -1 { return c; }
    c = coup_gagnant(g, "X");         // bloquer
    if c != -1 { return c; }
    if g[4] == "." { return 4; }      // centre
    return g.index_of(".");           // première case libre
}

println(choisir(["O","O",".","X",".",".","X",".","."]));   // gagner en 2
println(choisir(["X","X",".",".","O",".",".",".","."]));   // bloquer en 2
println(choisir(["X",".",".",".",".",".",".",".","."]));   // centre
