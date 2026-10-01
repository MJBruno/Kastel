// ==================================================================
// Exemple 386 — Toujours déverrouiller avec finally
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// Même si la section critique lève une erreur.
// ------------------------------------------------------------------
// Sortie attendue :
//   oups
//   false
// ==================================================================

let m = mutex();

func risquee() {
    m.lock();
    try {
        throw "oups";
    } finally {
        m.unlock();
    }
}

try {
    risquee();
} catch (e) {
    println(e);
}
println(m.is_locked());
