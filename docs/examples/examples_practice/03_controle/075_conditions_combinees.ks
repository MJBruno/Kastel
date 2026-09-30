// ==================================================================
// Exemple 075 — Combiner des conditions
// Catégorie : Structures de contrôle
// ------------------------------------------------------------------
// && (et), || (ou), ! (non) avec parenthèses pour clarifier.
// ------------------------------------------------------------------
// Sortie attendue :
//   tarif plein
// ==================================================================

let age = 30;
let etudiant = false;
if (age < 26 || etudiant) && age > 15 {
    println("tarif réduit");
} else {
    println("tarif plein");
}
