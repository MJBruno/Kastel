// ==================================================================
// Exemple 648 — Limiter la concurrence avec un sémaphore
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : 6 jobs, mais jamais plus de 2 en même temps.
// ------------------------------------------------------------------
// Sortie attendue :
//   6
//   true
// ==================================================================

let s = semaphore(2);
let actifs = 0;
let maximum = 0;
let termines = 0;

func job() {
    s.acquire();
    actifs += 1;
    maximum = max(maximum, actifs);
    yield();
    actifs -= 1;
    s.release();
    termines += 1;
}

let taches = [];
for i in range(6) {
    taches.add(spawn(job));
}
for t in taches {
    t.join();
}
println(termines);
println(maximum <= 2);
