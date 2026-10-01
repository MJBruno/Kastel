// ==================================================================
// Exemple 336 — Un range est une valeur
// Catégorie : Itérateurs et range
// ------------------------------------------------------------------
// range(a, b) se comporte comme une suite : size(), start(), stop()...
// ------------------------------------------------------------------
// Sortie attendue :
//   4
//   2
//   10
//   2
// ==================================================================

let r = range(2, 10, 2);
println(r.size());
println(r.start());
println(r.stop());
println(r.step());
