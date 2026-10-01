// ==================================================================
// Exemple 385 — try_lock
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// try_lock() prend le verrou s'il est libre, sinon renvoie false sans attendre.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   true
//   false
// ==================================================================

let m = mutex();
println(m.try_lock());
println(m.is_locked());
m.unlock();
println(m.is_locked());
