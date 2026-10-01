// ==================================================================
// Exemple 367 — Suivre l'état d'une tâche
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// status() : ready, running, waiting, done, cancelled...
// ------------------------------------------------------------------
// Sortie attendue :
//   done
//   true
// ==================================================================

func rien() {}

let t = spawn(rien);
t.join();
println(t.status());
println(t.is_done());
