// ==================================================================
// Exemple 134 — Curryfication
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Transformer une fonction à 2 arguments en deux fonctions à 1 argument.
// ------------------------------------------------------------------
// Sortie attendue :
//   42
//   42
// ==================================================================

let multiplier = a => b => a * b;
let fois3 = multiplier(3);
println(fois3(14));
println(multiplier(6)(7));
