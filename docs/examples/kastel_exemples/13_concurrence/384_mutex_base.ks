// ==================================================================
// Exemple 384 — Mutex : exclusion mutuelle
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// lock() / unlock() protègent une section critique.
// ------------------------------------------------------------------
// Sortie attendue :
//   200
// ==================================================================

let m = mutex();
let total = 0;

func ajouter() {
    for i in range(100) {
        m.lock();
        total += 1;
        m.unlock();
    }
}

let a = spawn(ajouter);
let b = spawn(ajouter);
a.join();
b.join();
println(total);
