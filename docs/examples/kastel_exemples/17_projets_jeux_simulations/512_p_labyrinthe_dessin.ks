// ==================================================================
// Exemple 512 — Dessiner une carte au trésor
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : afficher une carte avec le joueur (@) après quelques déplacements.
// ------------------------------------------------------------------
// Sortie attendue :
//   #####
//   #...#
//   #.#.#
//   #..@#
//   #####
// ==================================================================

let carte = [
    "#####",
    "#...#",
    "#.#.#",
    "#...#",
    "#####"
];
let x = 1;
let y = 1;

for ordre in "DDBB" {
    let nx = x;
    let ny = y;
    if ordre == "D" { nx += 1; }
    if ordre == "B" { ny += 1; }
    if carte[ny].char_at(nx) != "#" {
        x = nx;
        y = ny;
    }
}

for j in range(carte.size()) {
    let ligne = "";
    for i in range(5) {
        ligne += (i == x && j == y) ? "@" : carte[j].char_at(i);
    }
    println(ligne);
}
