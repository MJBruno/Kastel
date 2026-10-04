// ====================================================================
// Kastel — Opérateurs de comparaison
// Notions : < <= > >= == != ; entier et flottant se comparent
// Résultat attendu :
//   x < y  : true
//   x <= y : true
//   x > y  : false
//   x >= y : false
//   x == y : false
//   x != y : true
//   10 == 10.0 : true
// ====================================================================

let x = 10;
let y = 20;

println("x < y  : {}", x < y);
println("x <= y : {}", x <= y);
println("x > y  : {}", x > y);
println("x >= y : {}", x >= y);
println("x == y : {}", x == y);
println("x != y : {}", x != y);
println("10 == 10.0 : {}", 10 == 10.0);
