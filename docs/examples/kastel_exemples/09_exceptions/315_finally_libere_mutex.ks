// ==================================================================
// Exemple 315 — finally libère un verrou
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Le verrou est rendu même quand la tâche est annulée.
// ------------------------------------------------------------------
// Sortie attendue :
//   false
// ==================================================================

let m = mutex();
let porte = channel<int>();
let jamais = channel<int>();

func detenteur() {
    m.lock();
    try {
        porte.send(1);
        jamais.recv();
    } finally {
        m.unlock();
    }
}

let t = spawn(detenteur);
porte.recv();
t.cancel();
try {
    t.join();
} catch (e) {
}
println(m.is_locked());
