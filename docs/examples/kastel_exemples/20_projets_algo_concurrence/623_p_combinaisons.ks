// ==================================================================
// Exemple 623 — Toutes les combinaisons de k éléments
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : choisir 2 éléments parmi 4 sans ordre.
// ------------------------------------------------------------------
// Sortie attendue :
//   [[1, 2], [1, 3], [1, 4], [2, 3], [2, 4], [3, 4]]
// ==================================================================

func combiner(n: int, k: int, debut: int, courant: List<int>, res: List<List<int>>) {
    if courant.size() == k {
        res.add(courant.copy());
        return;
    }
    for i in range(debut, n + 1) {
        courant.add(i);
        combiner(n, k, i + 1, courant, res);
        courant.pop();
    }
}

let res = [];
combiner(4, 2, 1, [], res);
println(res);
