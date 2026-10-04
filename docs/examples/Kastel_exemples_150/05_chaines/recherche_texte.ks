// ====================================================================
// Kastel — Rechercher dans une chaîne
// Notions : contains, starts_with, ends_with, index_of, last_index_of
// Résultat attendu :
//   true
//   true
//   true
//   3
//   -1
//   7
// ====================================================================

let mot = "programmation";

println(mot.contains("gram"));
println(mot.starts_with("pro"));
println(mot.ends_with("ion"));
println(mot.index_of("gram"));
println(mot.index_of("xyz"));
println(mot.last_index_of("m"));
