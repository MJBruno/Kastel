// ====================================================================
// Kastel — Ensembles : les bases
// Notions : Set(...), add, contains, remove, size, unicité
// Résultat attendu :
//   4
//   true
//   false
//   3
// ====================================================================

let ensemble = Set(1, 2, 3);

ensemble.add(4);
ensemble.add(2);   // déjà présent : ignoré

println(ensemble.size());
println(ensemble.contains(3));

ensemble.remove(3);
println(ensemble.contains(3));
println(ensemble.size());
