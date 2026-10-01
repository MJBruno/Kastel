// ==================================================================
// Exemple 491 — Pong : rebonds d'une balle
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : une balle rebondit sur les bords d'un terrain de 10 de large.
// ------------------------------------------------------------------
// Sortie attendue :
//   x = 2, rebonds = 1
// ==================================================================

let x = 8;
let vx = 1;
let rebonds = 0;
for t in range(10) {
    x += vx;
    if x >= 10 || x <= 0 {
        vx = -vx;
        rebonds += 1;
    }
}
println("x = {}, rebonds = {}", x, rebonds);
