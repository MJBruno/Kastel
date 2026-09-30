// ==================================================================
// Exemple 389 — Limiter la concurrence
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// Chaque tâche prend un jeton, travaille, puis le rend.
// ------------------------------------------------------------------
// Sortie attendue :
//   5
//   2
// ==================================================================

let s = semaphore(2);
let fait = 0;

func job() {
    s.acquire();
    fait += 1;
    yield();
    s.release();
}

let taches = [];
for i in range(5) {
    taches.add(spawn(job));
}
for t in taches {
    t.join();
}
println(fait);
println(s.available());
