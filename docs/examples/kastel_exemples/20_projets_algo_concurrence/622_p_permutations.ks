// ==================================================================
// Exemple 622 — Toutes les permutations
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : retour arrière en choisissant chaque élément restant tour à tour.
// ------------------------------------------------------------------
// Sortie attendue :
//   6
//   [1, 2, 3]
//   [1, 3, 2]
//   [2, 1, 3]
//   [2, 3, 1]
//   [3, 1, 2]
//   [3, 2, 1]
// ==================================================================

func permuter(restants: List<int>, courant: List<int>, res: List<List<int>>) {
    if restants.is_empty() {
        res.add(courant.copy());
        return;
    }
    for i in range(restants.size()) {
        let reste = restants.copy();
        reste.remove_at(i);
        courant.add(restants[i]);
        permuter(reste, courant, res);
        courant.pop();
    }
}

let res = [];
permuter([1, 2, 3], [], res);
println(res.size());
for p in res { println(p); }
