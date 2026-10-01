// ==================================================================
// Exemple 062 — Fréquence de chaque lettre
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// Un dict sert de compteur : get_or(cle, defaut).
// ------------------------------------------------------------------
// Sortie attendue :
//   5
//   2
//   2
// ==================================================================

let compte = dict();
for c in "abracadabra" {
    compte[c] = compte.get_or(c, 0) + 1;
}
println(compte["a"]);   // 5
println(compte["b"]);   // 2
println(compte["r"]);   // 2
