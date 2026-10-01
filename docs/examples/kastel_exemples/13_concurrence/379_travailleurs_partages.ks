// ==================================================================
// Exemple 379 — Plusieurs travailleurs, un canal
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// Répartir des jobs ; le total ne dépend pas de l'ordre.
// ------------------------------------------------------------------
// Sortie attendue :
//   91
// ==================================================================

let jobs = channel<int>();
let resultats = channel<int>();

func travailleur() {
    for i in range(3) {
        let n = jobs.recv();
        resultats.send(n * n);
    }
}

let t1 = spawn(travailleur);
let t2 = spawn(travailleur);
for n in range(1, 7) {
    jobs.send(n);
}

let total = 0;
for i in range(6) {
    total += resultats.recv();
}
println(total);
