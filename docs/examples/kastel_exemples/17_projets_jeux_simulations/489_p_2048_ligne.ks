// ==================================================================
// Exemple 489 — 2048 : fusionner une ligne
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : glisser les tuiles vers la gauche et fusionner les paires égales une seule fois.
// ------------------------------------------------------------------
// Sortie attendue :
//   [4, 4, 0, 0]
//   [4, 4, 0, 0]
//   [8, 0, 0, 0]
//   [2, 4, 8, 16]
// ==================================================================

func fusionner(ligne: List<int>) -> List<int> {
    let vals = [];
    for v in ligne { if v != 0 { vals.add(v); } }

    let res = [];
    let i = 0;
    while i < vals.size() {
        if i + 1 < vals.size() && vals[i] == vals[i + 1] {
            res.add(vals[i] * 2);
            i += 2;
        } else {
            res.add(vals[i]);
            i += 1;
        }
    }
    while res.size() < ligne.size() { res.add(0); }
    return res;
}

println(fusionner([2, 2, 4, 0]));
println(fusionner([2, 2, 2, 2]));
println(fusionner([0, 4, 0, 4]));
println(fusionner([2, 4, 8, 16]));
