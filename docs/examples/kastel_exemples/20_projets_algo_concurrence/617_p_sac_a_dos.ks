// ==================================================================
// Exemple 617 — Problème du sac à dos 0/1
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : programmation dynamique, valeur maximale pour une capacité donnée.
// ------------------------------------------------------------------
// Sortie attendue :
//   9
// ==================================================================

func sac(poids: List<int>, valeurs: List<int>, capacite: int) -> int {
    let dp = [];
    for w in range(capacite + 1) { dp.add(0); }
    for i in range(poids.size()) {
        for w in range(capacite, poids[i] - 1, -1) {
            dp[w] = max(dp[w], dp[w - poids[i]] + valeurs[i]);
        }
    }
    return dp[capacite];
}

println(sac([1, 3, 4, 5], [1, 4, 5, 7], 7));
