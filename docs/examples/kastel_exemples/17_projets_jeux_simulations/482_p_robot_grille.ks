// ==================================================================
// Exemple 482 — Robot sur une grille
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : un robot suit des ordres A (avancer), G et D (tourner).
// ------------------------------------------------------------------
// Sortie attendue :
//   (2, 3)
// ==================================================================

let x = 0;
let y = 0;
let dir = 0;   // 0 nord, 1 est, 2 sud, 3 ouest

for o in "AADAAGA" {
    if o == "G" { dir = (dir + 3) % 4; }
    if o == "D" { dir = (dir + 1) % 4; }
    if o == "A" {
        if dir == 0 { y += 1; }
        if dir == 1 { x += 1; }
        if dir == 2 { y -= 1; }
        if dir == 3 { x -= 1; }
    }
}
println("({}, {})", x, y);
