// ==================================================================
// Exemple 511 — Carré magique
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : vérifier que lignes, colonnes et diagonales ont la même somme.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

func est_magique(m: List<List<int>>) -> bool {
    let n = m.size();
    let cible = 0;
    for x in m[0] { cible += x; }

    let d1 = 0;
    let d2 = 0;
    for i in range(n) {
        let l = 0;
        let c = 0;
        for j in range(n) {
            l += m[i][j];
            c += m[j][i];
        }
        if l != cible || c != cible { return false; }
        d1 += m[i][i];
        d2 += m[i][n - 1 - i];
    }
    return d1 == cible && d2 == cible;
}

println(est_magique([[2, 7, 6], [9, 5, 1], [4, 3, 8]]));
println(est_magique([[1, 2, 3], [4, 5, 6], [7, 8, 9]]));
