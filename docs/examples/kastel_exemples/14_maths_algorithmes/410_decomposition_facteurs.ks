// ==================================================================
// Exemple 410 — Décomposition en facteurs premiers
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Diviser tant que possible par chaque diviseur.
// ------------------------------------------------------------------
// Sortie attendue :
//   [2, 2, 2, 3, 3, 5]
// ==================================================================

func facteurs(n: int) -> List<int> {
    let res = [];
    let d = 2;
    while n > 1 {
        while n % d == 0 {
            res.add(d);
            n = idiv(n, d);
        }
        d += 1;
    }
    return res;
}

println(facteurs(360));
