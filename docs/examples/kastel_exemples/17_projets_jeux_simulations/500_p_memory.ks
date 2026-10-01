// ==================================================================
// Exemple 500 — Jeu de mémoire (paires)
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : retourner deux cartes, garder la paire si elles sont identiques.
// ------------------------------------------------------------------
// Sortie attendue :
//   paire A
//   raté : B / C
//   paire B
//   paire C
//   3
// ==================================================================

let cartes = ["A", "B", "A", "C", "B", "C"];
let essais = [(0, 2), (1, 3), (1, 4), (3, 5)];
let trouvees = Set();

for e in essais {
    if cartes[e[0]] == cartes[e[1]] {
        trouvees.add(cartes[e[0]]);
        println("paire " + cartes[e[0]]);
    } else {
        println("raté : " + cartes[e[0]] + " / " + cartes[e[1]]);
    }
}
println(trouvees.size());
