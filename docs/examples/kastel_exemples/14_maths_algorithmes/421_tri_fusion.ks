// ==================================================================
// Exemple 421 — Tri fusion
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Diviser, trier chaque moitié, fusionner.
// ------------------------------------------------------------------
// Sortie attendue :
//   [3, 9, 10, 27, 38, 43, 82]
// ==================================================================

func fusion(a: List<int>, b: List<int>) -> List<int> {
    let res = [];
    let i = 0;
    let j = 0;
    while i < a.size() && j < b.size() {
        if a[i] <= b[j] {
            res.add(a[i]);
            i += 1;
        } else {
            res.add(b[j]);
            j += 1;
        }
    }
    while i < a.size() { res.add(a[i]); i += 1; }
    while j < b.size() { res.add(b[j]); j += 1; }
    return res;
}

func tri_fusion(v: List<int>) -> List<int> {
    if v.size() <= 1 {
        return v;
    }
    let milieu = idiv(v.size(), 2);
    return fusion(tri_fusion(v.slice(0, milieu)), tri_fusion(v.slice(milieu, v.size())));
}

println(tri_fusion([38, 27, 43, 3, 9, 82, 10]));
