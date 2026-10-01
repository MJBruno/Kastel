// ==================================================================
// Exemple 625 — Bornes inférieure et supérieure
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : compter les occurrences d'une valeur dans une liste triée en O(log n).
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   4
//   3
// ==================================================================

// Premier indice i tel que v[i] >= x.
func borne_inf(v: List<int>, x: int) -> int {
    let bas = 0;
    let haut = v.size();
    while bas < haut {
        let m = idiv(bas + haut, 2);
        if v[m] < x { bas = m + 1; } else { haut = m; }
    }
    return bas;
}

// Premier indice i tel que v[i] > x.
func borne_sup(v: List<int>, x: int) -> int {
    let bas = 0;
    let haut = v.size();
    while bas < haut {
        let m = idiv(bas + haut, 2);
        if v[m] <= x { bas = m + 1; } else { haut = m; }
    }
    return bas;
}

let v = [1, 2, 2, 2, 3, 5];
println(borne_inf(v, 2));
println(borne_sup(v, 2));
println(borne_sup(v, 2) - borne_inf(v, 2));
