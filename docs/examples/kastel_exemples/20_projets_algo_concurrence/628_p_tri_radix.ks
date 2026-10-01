// ==================================================================
// Exemple 628 — Tri radix (par chiffres)
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : trier chiffre par chiffre, des unités vers les centaines.
// ------------------------------------------------------------------
// Sortie attendue :
//   [2, 24, 45, 66, 75, 90, 170, 802]
// ==================================================================

func radix(v: List<int>) -> List<int> {
    let res = v.copy();
    let maxi = 0;
    for x in res { maxi = max(maxi, x); }
    let exp = 1;
    while idiv(maxi, exp) > 0 {
        let seaux = [];
        for i in range(10) { seaux.add([]); }
        for x in res {
            seaux[idiv(x, exp) % 10].add(x);
        }
        res = [];
        for s in seaux {
            for x in s { res.add(x); }
        }
        exp *= 10;
    }
    return res;
}

println(radix([170, 45, 75, 90, 802, 24, 2, 66]));
