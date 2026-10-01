// ==================================================================
// Exemple 113 — Passer une fonction en paramètre
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Fonction d'ordre supérieur : appliquer deux fois une fonction.
// ------------------------------------------------------------------
// Sortie attendue :
//   16
// ==================================================================

func deux_fois(f, x) {
    return f(f(x));
}

func plus_trois(n: int) -> int {
    return n + 3;
}

println(deux_fois(plus_trois, 10));
