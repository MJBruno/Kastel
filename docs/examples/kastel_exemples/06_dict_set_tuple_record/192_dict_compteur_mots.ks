// ==================================================================
// Exemple 192 — Compter les mots
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// Un histogramme avec get_or.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
//   2
//   1
// ==================================================================

let texte = "un deux un trois un deux";
let compte = dict();
for mot in texte.split(" ") {
    compte[mot] = compte.get_or(mot, 0) + 1;
}
println(compte["un"]);
println(compte["deux"]);
println(compte["trois"]);
