// ====================================================================
// Kastel — Opérations ensemblistes
// Notions : union, intersection, difference, symmetric_difference, is_subset, is_superset
// Résultat attendu :
//   5
//   2
//   2
//   3
//   true
//   true
// ====================================================================

let a = Set(1, 2, 3, 4);
let b = Set(3, 4, 5);

println(a.union(b).size());
println(a.intersection(b).size());
println(a.difference(b).size());
println(a.symmetric_difference(b).size());
println(Set(1, 2).is_subset(a));
println(a.is_superset(Set(1, 2)));
