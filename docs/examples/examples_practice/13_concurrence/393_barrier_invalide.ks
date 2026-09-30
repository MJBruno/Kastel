// ==================================================================
// Exemple 393 — Barrier de taille invalide
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// barrier(0) est refusé.
// ------------------------------------------------------------------
// Sortie attendue :
//   BarrierNonPositive
// ==================================================================

try {
    let b = barrier(0);
} catch (e: Err) {
    println(e.kind);
}
