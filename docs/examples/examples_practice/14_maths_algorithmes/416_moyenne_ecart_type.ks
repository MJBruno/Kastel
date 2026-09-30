// ==================================================================
// Exemple 416 — Moyenne et écart-type
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Statistiques de base sur une liste.
// ------------------------------------------------------------------
// Sortie attendue :
//   5.0
//   2.0
// ==================================================================

let v = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
let somme = 0.0;
for x in v {
    somme += x;
}
let moyenne = somme / v.size();

let variance = 0.0;
for x in v {
    variance += (x - moyenne) * (x - moyenne);
}
variance = variance / v.size();

println(moyenne);
println(sqrt(variance));
