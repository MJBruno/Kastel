// ==================================================================
// Exemple 127 — Écrire soi-même map
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Appliquer une fonction à chaque élément d'une liste.
// ------------------------------------------------------------------
// Sortie attendue :
//   [10, 20, 30]
// ==================================================================

func appliquer(f, v) {
    let res = [];
    for x in v {
        res.add(f(x));
    }
    return res;
}

println(appliquer(x => x * 10, [1, 2, 3]));
