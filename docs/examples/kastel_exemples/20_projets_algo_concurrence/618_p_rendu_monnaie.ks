// ==================================================================
// Exemple 618 — Rendu de monnaie minimal
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : nombre minimal de pièces (programmation dynamique, valable même quand le glouton échoue).
// ------------------------------------------------------------------
// Sortie attendue :
//   6
//   2
// ==================================================================

func pieces(valeurs: List<int>, montant: int) -> int {
    let INF = 1000000;
    let dp = [0];
    for m in range(1, montant + 1) {
        let meilleur = INF;
        for v in valeurs {
            if v <= m && dp[m - v] + 1 < meilleur {
                meilleur = dp[m - v] + 1;
            }
        }
        dp.add(meilleur);
    }
    return dp[montant];
}

println(pieces([1, 5, 10, 25], 63));
println(pieces([1, 3, 4], 6));      // glouton donnerait 3 (4+1+1)
