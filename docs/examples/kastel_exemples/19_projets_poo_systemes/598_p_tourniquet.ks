// ==================================================================
// Exemple 598 — Ordonnanceur à tourniquet (round robin)
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : chaque tâche reçoit 2 unités de temps à tour de rôle.
// ------------------------------------------------------------------
// Sortie attendue :
//   C termine à t=6
//   A termine à t=7
//   B termine à t=10
// ==================================================================

let quantum = 2;
let noms = ["A", "B", "C"];
let restant = [3, 5, 2];
let file = [0, 1, 2];            // indices des tâches en attente
let horloge = 0;

while !file.is_empty() {
    let i = file.remove_at(0);
    let duree = min(quantum, restant[i]);
    horloge += duree;
    restant[i] = restant[i] - duree;
    if restant[i] == 0 {
        println("{} termine à t={}", noms[i], horloge);
    } else {
        file.add(i);
    }
}
