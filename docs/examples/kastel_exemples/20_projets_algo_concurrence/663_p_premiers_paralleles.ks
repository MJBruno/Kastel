// ==================================================================
// Exemple 663 — Compter les nombres premiers en parallèle
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : quatre tâches comptent chacune les premiers d'un quart de l'intervalle [0, 100[.
// ------------------------------------------------------------------
// Sortie attendue :
//   25
// ==================================================================

func est_premier(n: int) -> bool {
    if n < 2 { return false; }
    let d = 2;
    while d * d <= n {
        if n % d == 0 { return false; }
        d += 1;
    }
    return true;
}

func compter(debut: int, fin: int) -> int {
    let n = 0;
    for x in range(debut, fin) {
        if est_premier(x) { n += 1; }
    }
    return n;
}

let taches = [];
for k in range(4) {
    taches.add(spawn(compter, k * 25, (k + 1) * 25));
}

let total = 0;
for t in taches {
    total += t.join();
}
println(total);
