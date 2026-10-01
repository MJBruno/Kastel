// ==================================================================
// Exemple 499 — Dessiner un échiquier
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : alterner cases claires et foncées.
// ------------------------------------------------------------------
// Sortie attendue :
//   8 #.#.#.#.
//   7 .#.#.#.#
//   6 #.#.#.#.
//   5 .#.#.#.#
//   4 #.#.#.#.
//   3 .#.#.#.#
//   2 #.#.#.#.
//   1 .#.#.#.#
//     abcdefgh
// ==================================================================

for ligne in range(8, 0, -1) {
    let s = str(ligne) + " ";
    for col in range(8) {
        s += (ligne + col) % 2 == 0 ? "#" : ".";
    }
    println(s);
}
println("  abcdefgh");
