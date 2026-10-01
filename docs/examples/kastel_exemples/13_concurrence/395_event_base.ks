// ==================================================================
// Exemple 395 — Event : un signal
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// wait() attend que set() ait été appelé.
// ------------------------------------------------------------------
// Sortie attendue :
//   42
//   true
// ==================================================================

let e = event();

func attendre() -> int {
    e.wait();
    return 42;
}

let t = spawn(attendre);
e.set();
println(t.join());
println(e.is_set());
