// ==================================================================
// Exemple 609 — Planning Pomodoro
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : 4 cycles de travail de 25 min, pauses de 5 min, longue pause de 15 min à la fin.
// ------------------------------------------------------------------
// Sortie attendue :
//   travail 1 fini à 25 min
//   travail 2 fini à 55 min
//   travail 3 fini à 85 min
//   travail 4 fini à 115 min
//   durée totale : 130 min
// ==================================================================

let temps = 0;
for cycle in range(1, 5) {
    temps += 25;
    println("travail {} fini à {} min", cycle, temps);
    if cycle < 4 {
        temps += 5;
    } else {
        temps += 15;
    }
}
println("durée totale : {} min", temps);
