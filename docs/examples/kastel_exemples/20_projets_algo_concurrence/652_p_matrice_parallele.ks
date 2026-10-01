// ==================================================================
// Exemple 652 — Produit de matrices : une tâche par ligne
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : chaque ligne du résultat est calculée indépendamment.
// ------------------------------------------------------------------
// Sortie attendue :
//   [[19, 22], [43, 50]]
// ==================================================================

func ligne(a_ligne: List<int>, b: List<List<int>>) -> List<int> {
    let res = [];
    for j in range(b[0].size()) {
        let s = 0;
        for k in range(a_ligne.size()) {
            s += a_ligne[k] * b[k][j];
        }
        res.add(s);
    }
    return res;
}

let a = [[1, 2], [3, 4]];
let b = [[5, 6], [7, 8]];

let taches = [];
for l in a {
    taches.add(spawn(ligne, l, b));
}

let resultat = [];
for t in taches {
    resultat.add(t.join());
}
println(resultat);
