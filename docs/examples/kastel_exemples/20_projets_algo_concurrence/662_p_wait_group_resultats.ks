// ==================================================================
// Exemple 662 — WaitGroup : collecter des résultats
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : 4 tâches ajoutent leur résultat à une liste partagée, le programme attend la fin du groupe.
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 4, 9, 16]
// ==================================================================

let wg = wait_group();
let resultats = [];

func job(n: int) {
    resultats.add(n * n);
    wg.done();
}

wg.add(4);
for i in range(1, 5) {
    spawn(job, i);
}
wg.wait();
resultats.sort();
println(resultats);
