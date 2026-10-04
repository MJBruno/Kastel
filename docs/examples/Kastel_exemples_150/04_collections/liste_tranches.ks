// ====================================================================
// Kastel — Tranches et inversion
// Notions : slice(debut, fin), reverse() en place
// Résultat attendu :
//   [20, 30]
//   [50, 40, 30, 20, 10]
// ====================================================================

let a = [10, 20, 30, 40, 50];

println(a.slice(1, 3));   // fin exclue

a.reverse();
println(a);
