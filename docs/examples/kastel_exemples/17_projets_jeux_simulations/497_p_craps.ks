// ==================================================================
// Exemple 497 — Craps : règles du jeu
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : 7 ou 11 au premier lancer gagne, 2/3/12 perd, sinon il faut refaire le point avant un 7.
// ------------------------------------------------------------------
// Sortie attendue :
//   gagné
//   perdu
//   gagné
//   perdu
// ==================================================================

func partie(lancers: List<int>) -> str {
    let premier = lancers[0];
    if premier == 7 || premier == 11 { return "gagné"; }
    if premier == 2 || premier == 3 || premier == 12 { return "perdu"; }
    for i in range(1, lancers.size()) {
        if lancers[i] == premier { return "gagné"; }
        if lancers[i] == 7 { return "perdu"; }
    }
    return "en cours";
}

println(partie([7]));
println(partie([2]));
println(partie([6, 3, 6]));
println(partie([6, 4, 7]));
