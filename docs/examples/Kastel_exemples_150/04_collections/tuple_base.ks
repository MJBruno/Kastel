// ====================================================================
// Kastel — Tuples
// Notions : taille, get, first, last, contains, tuple à un élément
// Résultat attendu :
//   3
//   20
//   10
//   30
//   true
//   1
// ====================================================================

let t = (10, 20, 30);

println(t.size());
println(t.get(1));
println(t.first());
println(t.last());
println(t.contains(20));

let seul = (5,);   // la virgule distingue un tuple d'une parenthèse
println(seul.size());
