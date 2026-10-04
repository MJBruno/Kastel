// ====================================================================
// Kastel — Arithmétique cyclique
// Notions : wrapping_add, wrapping_sub, wrapping_mul
// Résultat attendu :
//   -9223372036854775808
//   -2
//   9223372036854775807
// ====================================================================

let max = 9223372036854775807;
let min = -9223372036854775807 - 1;

println(wrapping_add(max, 1));
println(wrapping_mul(max, 2));
println(wrapping_sub(min, 1));
