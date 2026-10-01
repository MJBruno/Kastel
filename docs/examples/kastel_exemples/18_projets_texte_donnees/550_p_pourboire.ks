// ==================================================================
// Exemple 550 — Calculateur de pourboire
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : additionner addition et pourboire, puis partager entre convives.
// ------------------------------------------------------------------
// Sortie attendue :
//   pourboire : 12.00
//   total : 92.00
//   par personne : 23.00
// ==================================================================

let addition = 80.0;
let pourcentage = 15.0;
let convives = 4;

let pourboire = addition * pourcentage / 100.0;
let total = addition + pourboire;
println("pourboire : {:.2f}", pourboire);
println("total : {:.2f}", total);
println("par personne : {:.2f}", total / convives);
