// ==================================================================
// Exemple 505 — Feu tricolore avec durées
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : cycle rouge, vert, orange avec un compteur de secondes.
// ------------------------------------------------------------------
// Sortie attendue :
//   t= 0 : rouge
//   t= 3 : vert
//   t= 6 : orange
//   t= 7 : rouge
//   t=10 : vert
//   t=13 : orange
// ==================================================================

let phases = [("rouge", 3), ("vert", 3), ("orange", 1)];
let temps = 0;
for cycle in range(2) {
    for p in phases {
        println("t={:>2} : {}", temps, p[0]);
        temps += p[1];
    }
}
