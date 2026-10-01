// ==================================================================
// Exemple 504 — Simulation d'ascenseur
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : l'ascenseur traite des demandes d'étages dans l'ordre et compte les étages parcourus.
// ------------------------------------------------------------------
// Sortie attendue :
//   de 0 à 5 (5 étages)
//   de 5 à 2 (3 étages)
//   de 2 à 8 (6 étages)
//   de 8 à 0 (8 étages)
//   total : 22
// ==================================================================

let etage = 0;
let parcourus = 0;
for demande in [5, 2, 8, 0] {
    let distance = abs(demande - etage);
    parcourus += distance;
    println("de {} à {} ({} étages)", etage, demande, distance);
    etage = demande;
}
println("total : {}", parcourus);
