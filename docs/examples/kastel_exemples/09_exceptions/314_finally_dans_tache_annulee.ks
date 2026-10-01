// ==================================================================
// Exemple 314 — finally et annulation de tâche
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Annuler une tâche exécute ses blocs finally, mais pas ses catch.
// ------------------------------------------------------------------
// Sortie attendue :
//   ["finally"]
//   cancelled
// ==================================================================

let journal = [];
let porte = channel<int>();

func travailleur() {
    try {
        porte.send(1);
        while true {
            yield();
        }
    } catch (e) {
        journal.add("catch");
    } finally {
        journal.add("finally");
    }
}

let t = spawn(travailleur);
porte.recv();
t.cancel();
try {
    t.join();
} catch (e) {
}
println(journal);
println(t.status());
