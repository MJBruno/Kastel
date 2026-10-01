// ==================================================================
// Exemple 390 — WaitGroup : attendre un groupe
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// add(n) annonce n tâches, done() en termine une, wait() attend qu'il n'en reste aucune.
// ------------------------------------------------------------------
// Sortie attendue :
//   6
// ==================================================================

let wg = wait_group();
let total = 0;

func job(n: int) {
    total += n;
    wg.done();
}

wg.add(3);
spawn(job, 1);
spawn(job, 2);
spawn(job, 3);
wg.wait();
println(total);
