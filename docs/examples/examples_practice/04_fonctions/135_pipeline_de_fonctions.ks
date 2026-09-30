// ==================================================================
// Exemple 135 — Pipeline de transformations
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Enchaîner une liste de fonctions sur une valeur.
// ------------------------------------------------------------------
// Sortie attendue :
//   35
// ==================================================================

func pipeline(valeur, fonctions) {
    let v = valeur;
    for f in fonctions {
        v = f(v);
    }
    return v;
}

println(pipeline(3, [x => x + 1, x => x * 10, x => x - 5]));
