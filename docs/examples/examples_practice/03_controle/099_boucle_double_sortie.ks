// ==================================================================
// Exemple 099 — Sortir de deux boucles
// Catégorie : Structures de contrôle
// ------------------------------------------------------------------
// Un drapeau booléen permet d'arrêter la boucle externe.
// ------------------------------------------------------------------
// Sortie attendue :
//   6 x 7
// ==================================================================

let trouve = false;
for i in range(1, 10) {
    for j in range(1, 10) {
        if i * j == 42 {
            println("{} x {}", i, j);
            trouve = true;
            break;
        }
    }
    if trouve {
        break;
    }
}
