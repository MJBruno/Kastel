// ==================================================================
// Exemple 398 — Annuler une tâche
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// cancel() arrête une tâche ; join() lève alors TaskCancelled.
// ------------------------------------------------------------------
// Sortie attendue :
//   TaskCancelled
//   cancelled
// ==================================================================

func longue() -> int {
    sleep(10000);
    return 1;
}

let t = spawn(longue);
t.cancel();
try {
    t.join();
} catch (e: Err) {
    println(e.kind);
}
println(t.status());
