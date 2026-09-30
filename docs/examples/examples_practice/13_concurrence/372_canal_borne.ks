// ==================================================================
// Exemple 372 — Canal borné
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// channel<T>(n) accepte au plus n messages en attente ; capacity() et is_full() renseignent.
// ------------------------------------------------------------------
// Sortie attendue :
//   Some(2)
//   true
//   2
//   false
// ==================================================================

let c = channel<int>(2);
println(c.capacity());
c.send(1);
c.send(2);
println(c.is_full());
println(c.size());
c.recv();
println(c.is_full());
