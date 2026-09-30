// ==================================================================
// Exemple 340 — peek : regarder sans avancer
// Catégorie : Itérateurs et range
// ------------------------------------------------------------------
// peek() renvoie la prochaine valeur sans la consommer.
// ------------------------------------------------------------------
// Sortie attendue :
//   10
//   10
//   20
//   false
// ==================================================================

let it = [10, 20].iter();
println(it.peek());
println(it.next());
println(it.next());
println(it.has_next());
