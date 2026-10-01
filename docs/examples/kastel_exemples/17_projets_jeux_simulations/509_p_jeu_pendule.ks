// ==================================================================
// Exemple 509 — Chasse au trésor sur grille
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : suivre des indices (nord, sud, est, ouest) jusqu'au trésor.
// ------------------------------------------------------------------
// Sortie attendue :
//   trésor en (2, 2)
// ==================================================================

let indices = ["N3", "E4", "S1", "O2"];
let x = 0;
let y = 0;
for ind in indices {
    let dir = ind.char_at(0);
    let pas = ind.slice(1, ind.size()).to_int();
    match dir {
        "N" => { y += pas; }
        "S" => { y -= pas; }
        "E" => { x += pas; }
        _ => { x -= pas; }
    }
}
println("trésor en ({}, {})", x, y);
