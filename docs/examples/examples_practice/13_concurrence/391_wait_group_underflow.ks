// ==================================================================
// Exemple 391 — done() en trop
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// Terminer plus de tâches que prévu est une erreur.
// ------------------------------------------------------------------
// Sortie attendue :
//   WaitGroupUnderflow
// ==================================================================

let wg = wait_group();
try {
    wg.done();
} catch (e: Err) {
    println(e.kind);
}
