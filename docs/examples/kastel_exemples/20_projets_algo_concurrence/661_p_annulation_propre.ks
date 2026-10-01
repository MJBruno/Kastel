// ==================================================================
// Exemple 661 — Annulation propre de tâches
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : on annule trois tâches bloquées ; leurs finally libèrent chacune leur ressource.
// ------------------------------------------------------------------
// Sortie attendue :
//   0
// ==================================================================

let ouvertes = 0;
let jamais = channel<int>();

func travailleur() {
    ouvertes += 1;
    try {
        jamais.recv();
    } finally {
        ouvertes -= 1;
    }
}

let taches = [];
for i in range(3) {
    taches.add(spawn(travailleur));
}
while ouvertes < 3 {
    yield();
}
for t in taches {
    t.cancel();
}
for t in taches {
    try {
        t.join();
    } catch (e) {
    }
}
println(ouvertes);
