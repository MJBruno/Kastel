// ==================================================================
// Exemple 626 — k-ième plus petit élément
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : partitionner autour d'un pivot sans trier toute la liste.
// ------------------------------------------------------------------
// Sortie attendue :
//   7
//   3
// ==================================================================

func kieme(v: List<int>, k: int) -> int {
    let pivot = v[0];
    let petits = [];
    let egaux = [];
    let grands = [];
    for x in v {
        if x < pivot { petits.add(x); }
        else if x == pivot { egaux.add(x); }
        else { grands.add(x); }
    }
    if k <= petits.size() { return kieme(petits, k); }
    if k <= petits.size() + egaux.size() { return pivot; }
    return kieme(grands, k - petits.size() - egaux.size());
}

println(kieme([7, 10, 4, 3, 20, 15], 3));
println(kieme([7, 10, 4, 3, 20, 15], 1));
