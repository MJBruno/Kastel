// ==================================================================
// Exemple 643 — Somme parallèle
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : découper 1..100 en 4 morceaux additionnés par 4 tâches.
// ------------------------------------------------------------------
// Sortie attendue :
//   5050
// ==================================================================

func somme(v: List<int>) -> int {
    let s = 0;
    for x in v { s += x; }
    return s;
}

let nombres = [];
for i in range(1, 101) { nombres.add(i); }

let taches = [];
for k in range(4) {
    taches.add(spawn(somme, nombres.slice(k * 25, (k + 1) * 25)));
}

let total = 0;
for t in taches {
    total += t.join();
}
println(total);
