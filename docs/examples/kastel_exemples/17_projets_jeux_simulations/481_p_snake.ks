// ==================================================================
// Exemple 481 — Snake : déplacer le serpent
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : appliquer une suite de commandes et détecter une collision avec le mur.
// ------------------------------------------------------------------
// Sortie attendue :
//   collision en (5, 0)
//   false
// ==================================================================

let taille = 5;
let x = 2;
let y = 2;
let commandes = ["H", "H", "D", "D", "D", "B"];

let vivant = true;
for c in commandes {
    if c == "H" { y -= 1; }
    if c == "B" { y += 1; }
    if c == "G" { x -= 1; }
    if c == "D" { x += 1; }
    if x < 0 || x >= taille || y < 0 || y >= taille {
        println("collision en ({}, {})", x, y);
        vivant = false;
        break;
    }
}
println(vivant);
