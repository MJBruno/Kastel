// ==================================================================
// Exemple 392 — Barrier : rendez-vous
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// Toutes les tâches attendent que les n soient arrivées.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
//   3
// ==================================================================

let b = barrier(3);
let arrivees = 0;
let apres = 0;

func participant() {
    arrivees += 1;
    b.wait();
    apres += 1;
}

let t1 = spawn(participant);
let t2 = spawn(participant);
let t3 = spawn(participant);
t1.join();
t2.join();
t3.join();
println(arrivees);
println(apres);
