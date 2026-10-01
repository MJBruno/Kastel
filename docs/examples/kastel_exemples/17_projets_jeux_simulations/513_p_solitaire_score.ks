// ==================================================================
// Exemple 513 — Solitaire : score d'une partie
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : total des points selon les cartes déplacées vers les fondations.
// ------------------------------------------------------------------
// Sortie attendue :
//   38
// ==================================================================

let points = {"as": 1, "roi": 13, "dame": 12, "valet": 11};
let coups = ["as", "roi", "valet", "as", "dame"];
let total = 0;
for c in coups {
    total += points.get_or(c, 0);
}
println(total);
