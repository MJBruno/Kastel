// ==================================================================
// Exemple 624 — Solveur de Sudoku 4 x 4
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : retour arrière sur les cases vides (0).
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   [1, 2, 3, 4]
//   [3, 4, 1, 2]
//   [2, 1, 4, 3]
//   [4, 3, 2, 1]
// ==================================================================

let g = [[0, 2, 3, 4],
         [3, 4, 1, 2],
         [2, 1, 4, 3],
         [4, 3, 2, 0]];

func possible(r: int, c: int, v: int) -> bool {
    for i in range(4) {
        if g[r][i] == v || g[i][c] == v { return false; }
    }
    let br = idiv(r, 2) * 2;
    let bc = idiv(c, 2) * 2;
    for i in range(2) {
        for j in range(2) {
            if g[br + i][bc + j] == v { return false; }
        }
    }
    return true;
}

func resoudre() -> bool {
    for r in range(4) {
        for c in range(4) {
            if g[r][c] == 0 {
                for v in range(1, 5) {
                    if possible(r, c, v) {
                        g[r][c] = v;
                        if resoudre() { return true; }
                        g[r][c] = 0;
                    }
                }
                return false;
            }
        }
    }
    return true;
}

println(resoudre());
for ligne in g { println(ligne); }
