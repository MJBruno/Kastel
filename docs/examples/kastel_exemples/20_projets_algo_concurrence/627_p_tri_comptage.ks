// ==================================================================
// Exemple 627 — Tri par comptage
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : trier des petits entiers en comptant leurs occurrences.
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 2, 2, 3, 3, 4, 8]
// ==================================================================

func tri_comptage(v: List<int>) -> List<int> {
    let maxi = 0;
    for x in v { maxi = max(maxi, x); }
    let compte = [];
    for i in range(maxi + 1) { compte.add(0); }
    for x in v { compte[x] += 1; }

    let res = [];
    for i in range(maxi + 1) {
        for k in range(compte[i]) {
            res.add(i);
        }
    }
    return res;
}

println(tri_comptage([4, 2, 2, 8, 3, 3, 1]));
