// ==================================================================
// Exemple 645 — Pool de travailleurs
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : 3 travailleurs se partagent 6 jobs via un canal ; on additionne les cubes.
// ------------------------------------------------------------------
// Sortie attendue :
//   441
// ==================================================================

let jobs = channel<int>();
let resultats = channel<int>();

func travailleur() {
    for i in range(2) {
        let n = jobs.recv();
        resultats.send(n * n * n);
    }
}

for i in range(3) {
    spawn(travailleur);
}
for n in range(1, 7) {
    jobs.send(n);
}

let total = 0;
for i in range(6) {
    total += resultats.recv();
}
println(total);
