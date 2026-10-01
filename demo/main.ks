// ==================================================================
// Exemple 394 — RwLock : lecteurs multiples, un écrivain
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// read_lock partagé, write_lock exclusif.
// ------------------------------------------------------------------
// Sortie attendue :
//   2
//   true
//   false
// ==================================================================

let l = rwlock();
l.read_lock();
l.read_lock();
println(l.reader_count());
l.read_unlock();
l.read_unlock();
l.write_lock();
println(l.is_write_locked());
l.write_unlock();
println(l.is_write_locked());
