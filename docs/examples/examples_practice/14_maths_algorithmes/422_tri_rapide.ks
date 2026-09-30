// ==================================================================
// Exemple 422 — Tri rapide
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Choisir un pivot, séparer plus petits et plus grands, recommencer.
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 2, 2, 3, 6, 8, 9]
// ==================================================================

func tri_rapide(v: List<int>) -> List<int> {
    if v.size() <= 1 {
        return v;
    }
    let pivot = v[0];
    let petits = [];
    let grands = [];
    for i in range(1, v.size()) {
        if v[i] < pivot {
            petits.add(v[i]);
        } else {
            grands.add(v[i]);
        }
    }
    let res = tri_rapide(petits);
    res.add(pivot);
    for x in tri_rapide(grands) {
        res.add(x);
    }
    return res;
}

println(tri_rapide([3, 6, 1, 8, 2, 9, 2]));
