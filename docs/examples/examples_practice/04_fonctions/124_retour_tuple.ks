// ==================================================================
// Exemple 124 — Retourner plusieurs valeurs
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Un tuple regroupe plusieurs résultats.
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   9
// ==================================================================

func min_max(v: List<int>) -> Tuple<int, int> {
    let petit = v[0];
    let grand = v[0];
    for x in v {
        if x < petit { petit = x; }
        if x > grand { grand = x; }
    }
    return (petit, grand);
}

let r = min_max([4, 9, 1, 7]);
println(r[0]);
println(r[1]);
