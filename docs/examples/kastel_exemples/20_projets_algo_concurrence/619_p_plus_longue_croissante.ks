// ==================================================================
// Exemple 619 — Plus longue sous-suite croissante
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : dp[i] = longueur de la meilleure suite qui finit en i.
// ------------------------------------------------------------------
// Sortie attendue :
//   4
// ==================================================================

func lis(v: List<int>) -> int {
    let dp = [];
    for x in v { dp.add(1); }
    let meilleur = 0;
    for i in range(v.size()) {
        for j in range(i) {
            if v[j] < v[i] && dp[j] + 1 > dp[i] {
                dp[i] = dp[j] + 1;
            }
        }
        meilleur = max(meilleur, dp[i]);
    }
    return meilleur;
}

println(lis([10, 9, 2, 5, 3, 7, 101, 18]));
