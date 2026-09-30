// ==================================================================
// Exemple 070 — Nettoyer une saisie
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// Chaîner plusieurs méthodes : trim, lower, replace_all.
// ------------------------------------------------------------------
// Sortie attendue :
//   jean dupont
// ==================================================================

let saisie = "  Jean  Dupont ";
let propre = saisie.trim().lower().replace_all("  ", " ");
println(propre);
