// ==================================================================
// Exemple 128 — Fonction prédicat
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Une fonction booléenne, réutilisable avec filter.
// ------------------------------------------------------------------
// Sortie attendue :
//   [2, 4, 6]
// ==================================================================

func est_pair(n: int) -> bool {
    return n % 2 == 0;
}

println([1, 2, 3, 4, 5, 6].filter(est_pair));
