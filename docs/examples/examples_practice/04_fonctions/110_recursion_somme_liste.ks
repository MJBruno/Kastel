// ==================================================================
// Exemple 110 — Récursion sur une liste
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Somme = premier élément + somme du reste.
// ------------------------------------------------------------------
// Sortie attendue :
//   15
// ==================================================================

func somme(v: List<int>) -> int {
    if v.is_empty() {
        return 0;
    }
    return v[0] + somme(v.slice(1, v.size()));
}

println(somme([1, 2, 3, 4, 5]));
