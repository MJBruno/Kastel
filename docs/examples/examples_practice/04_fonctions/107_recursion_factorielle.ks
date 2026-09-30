// ==================================================================
// Exemple 107 — Récursion : factorielle
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Une fonction qui s'appelle elle-même avec un cas d'arrêt.
// ------------------------------------------------------------------
// Sortie attendue :
//   120
//   3628800
// ==================================================================

func fact(n: int) -> int {
    if n <= 1 {
        return 1;
    }
    return n * fact(n - 1);
}

println(fact(5));
println(fact(10));
