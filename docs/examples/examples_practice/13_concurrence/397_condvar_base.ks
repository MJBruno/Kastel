// ==================================================================
// Exemple 397 — Variable de condition
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// wait() suspend la tâche jusqu'à notify_one() / notify_all().
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   réveillé
// ==================================================================

let m = mutex();
let cv = condvar(m);

func attente() -> str {
    m.lock();
    cv.wait();
    m.unlock();
    return "réveillé";
}

let t = spawn(attente);
while t.status() != "waiting" {
    sleep(1);
}
println(cv.waiter_count());
cv.notify_one();
println(t.join());
