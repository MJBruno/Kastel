// ==================================================================
// Exemple 394 — RwLock : lecteurs multiples, un écrivain
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// read_lock partagé, write_lock exclusif.
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   2
//   true
//   false
// ==================================================================

let l = rwlock();

let reader1 = spawn(func(lock) {
    lock.read_lock();

    println(lock.reader_count());

    yield();

    lock.read_unlock();
}, l);

let reader2 = spawn(func(lock) {
    lock.read_lock();

    println(lock.reader_count());

    yield();

    lock.read_unlock();
}, l);

reader1.join();
reader2.join();

let writer = spawn(func(lock) {
    lock.write_lock();

    println(lock.is_write_locked());

    lock.write_unlock();

    println(lock.is_write_locked());
}, l);

writer.join();