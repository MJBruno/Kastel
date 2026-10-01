// ==================================================================
// Exemple 485 — Taquin : la grille est-elle solvable ?
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : compter les inversions (règle pour une grille de largeur impaire).
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

func inversions(v: List<int>) -> int {
    let n = 0;
    for i in range(v.size()) {
        for j in range(i + 1, v.size()) {
            if v[i] != 0 && v[j] != 0 && v[i] > v[j] { n += 1; }
        }
    }
    return n;
}

func solvable(v: List<int>) -> bool {
    return inversions(v) % 2 == 0;
}

println(solvable([1, 2, 3, 4, 5, 6, 7, 8, 0]));
println(solvable([1, 2, 3, 4, 5, 6, 8, 7, 0]));
