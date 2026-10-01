// ==================================================================
// Exemple 502 — Tournoi à élimination
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : on garde le plus fort de chaque paire jusqu'au vainqueur.
// ------------------------------------------------------------------
// Sortie attendue :
//   tour 1 : 4 qualifiés
//   tour 2 : 2 qualifiés
//   tour 3 : 1 qualifiés
//   vainqueur : Ada
// ==================================================================

let joueurs = [("Ada", 9), ("Bob", 4), ("Cléo", 7), ("Dan", 8), ("Eve", 3), ("Fay", 6), ("Gus", 5), ("Hal", 2)];
let tour = 1;
while joueurs.size() > 1 {
    let suivants = [];
    let i = 0;
    while i < joueurs.size() {
        let a = joueurs[i];
        let b = joueurs[i + 1];
        suivants.add(a[1] >= b[1] ? a : b);
        i += 2;
    }
    joueurs = suivants;
    println("tour {} : {} qualifiés", tour, joueurs.size());
    tour += 1;
}
println("vainqueur : " + joueurs[0][0]);
