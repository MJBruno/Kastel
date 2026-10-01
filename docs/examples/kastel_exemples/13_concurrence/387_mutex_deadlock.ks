// ==================================================================
// Exemple 387 — Détection de blocage sur soi-même
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// Reverrouiller un mutex déjà détenu par la même tâche lève MutexDeadlock.
// ------------------------------------------------------------------
// Sortie attendue :
//   MutexDeadlock
// ==================================================================

let m = mutex();

func travailleur() -> str {
    m.lock();
    try {
        m.lock();
        return "inattendu";
    } catch (e: Err) {
        m.unlock();
        return e.kind;
    }
}

println(spawn(travailleur).join());
