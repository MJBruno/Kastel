// ==================================================================
// Exemple 132 — Récursion trop profonde
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Une récursion infinie lève StackOverflow au lieu de planter : on peut l'attraper.
// ------------------------------------------------------------------
// Sortie attendue :
//   StackOverflow
// ==================================================================

func boucle(n: int) -> int {
    return boucle(n + 1);
}

try {
    boucle(0);
} catch (e: Err) {
    println(e.kind);
}
