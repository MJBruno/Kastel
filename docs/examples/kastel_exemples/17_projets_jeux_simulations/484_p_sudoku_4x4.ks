// ==================================================================
// Exemple 484 — Sudoku 4 x 4 : valider une grille
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : chaque ligne, colonne et bloc 2 x 2 doit contenir 1 à 4.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

func valide_groupe(v: List<int>) -> bool {
    let copie = v.copy();
    copie.sort();
    return copie == [1, 2, 3, 4];
}

func valide(g: List<List<int>>) -> bool {
    for i in range(4) {
        let col = [];
        for j in range(4) { col.add(g[j][i]); }
        if !valide_groupe(g[i]) || !valide_groupe(col) { return false; }
    }
    for bi in range(0, 4, 2) {
        for bj in range(0, 4, 2) {
            let bloc = [g[bi][bj], g[bi][bj + 1], g[bi + 1][bj], g[bi + 1][bj + 1]];
            if !valide_groupe(bloc) { return false; }
        }
    }
    return true;
}

let bonne = [[1,2,3,4],[3,4,1,2],[2,1,4,3],[4,3,2,1]];
let mauvaise = [[1,2,3,4],[3,4,1,2],[2,1,4,3],[4,3,1,2]];
println(valide(bonne));
println(valide(mauvaise));
