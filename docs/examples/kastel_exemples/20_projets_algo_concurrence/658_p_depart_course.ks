// ==================================================================
// Exemple 658 — Départ de course avec un event
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : les coureurs attendent le signal, puis partent tous ensemble.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
// ==================================================================

let depart = event();
let arrives = [];

func coureur(numero: int) {
    depart.wait();
    arrives.add(numero);
}

let taches = [];
for i in range(3) {
    taches.add(spawn(coureur, i));
}
yield();
depart.set();
for t in taches {
    t.join();
}
println(arrives.size());
