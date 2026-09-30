// ==================================================================
// Exemple 388 — Sémaphore
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// semaphore(n) autorise au plus n tâches simultanées.
// ------------------------------------------------------------------
// Sortie attendue :
//   2
//   1
//   false
//   1
// ==================================================================

let s = semaphore(2);
println(s.available());
s.acquire();
println(s.available());
s.acquire();
println(s.try_acquire());
s.release();
println(s.available());
