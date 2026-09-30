// ==================================================================
// Exemple 068 — Encadrer un titre
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// repeat() pour dessiner des lignes de tirets.
// ------------------------------------------------------------------
// Sortie attendue :
//   ----------
//   | KASTEL |
//   ----------
// ==================================================================

let titre = "KASTEL";
let ligne = "-".repeat(titre.size() + 4);
println(ligne);
println("| " + titre + " |");
println(ligne);
