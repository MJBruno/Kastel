// ==================================================================
// Exemple 650 — Phases synchronisées par une barrière
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : aucune tâche ne commence la phase 2 avant que toutes aient fini la phase 1.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
// ==================================================================

let b = barrier(3);
let phase1 = 0;
let ok = true;

func participant() {
    phase1 += 1;
    b.wait();
    if phase1 != 3 {
        ok = false;      // la phase 2 a démarré trop tôt
    }
}

let taches = [];
for i in range(3) {
    taches.add(spawn(participant));
}
for t in taches {
    t.join();
}
println(ok);
