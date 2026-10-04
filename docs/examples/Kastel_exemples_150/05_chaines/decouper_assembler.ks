// ====================================================================
// Kastel — Découper et assembler
// Notions : split, join, size
// Résultat attendu :
//   3
//   a-b-c
// ====================================================================

let morceaux = "a,b,c".split(",");

println(morceaux.size());
println(morceaux.join("-"));
