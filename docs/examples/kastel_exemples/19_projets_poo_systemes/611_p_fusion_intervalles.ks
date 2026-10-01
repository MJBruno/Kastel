// ==================================================================
// Exemple 611 — Fusionner des intervalles
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : regrouper les intervalles qui se chevauchent (liste déjà triée par début).
// ------------------------------------------------------------------
// Sortie attendue :
//   [[1, 6], [8, 10], [15, 18]]
// ==================================================================

func fusionner(v: List<List<int>>) -> List<List<int>> {
    let res = [];
    for iv in v {
        if !res.is_empty() && iv[0] <= res[res.size() - 1][1] {
            let dernier = res[res.size() - 1];
            dernier[1] = max(dernier[1], iv[1]);
        } else {
            res.add([iv[0], iv[1]]);
        }
    }
    return res;
}

println(fusionner([[1, 3], [2, 6], [8, 10], [15, 18]]));
