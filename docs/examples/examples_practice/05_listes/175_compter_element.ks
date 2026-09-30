// ==================================================================
// Exemple 175 — Compter les occurrences
// Catégorie : Listes
// ------------------------------------------------------------------
// Un petit histogramme dans un dict.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
//   2
//   1
// ==================================================================

let couleurs = ["rouge", "bleu", "rouge", "vert", "rouge", "bleu"];
let compte = dict();
for c in couleurs {
    compte[c] = compte.get_or(c, 0) + 1;
}
println(compte["rouge"]);
println(compte["bleu"]);
println(compte["vert"]);
