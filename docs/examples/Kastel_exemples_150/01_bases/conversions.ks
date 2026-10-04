// ====================================================================
// Kastel — Conversions de types
// Notions : to_int, to_float, str, float, floor, ceil, round
// Résultat attendu :
//   42 + 1 = 43
//   3.5 * 2 = 7.0
//   str(123) + "!" = 123!
//   floor(3.7) = 3
//   ceil(3.2) = 4
//   round(2.6) = 3
//   float(3) = 3.0
// ====================================================================

let texte = "42";
let n = texte.to_int();
let f = "3.5".to_float();

println("{} + 1 = {}", n, n + 1);
println("{} * 2 = {}", f, f * 2);
println("str(123) + \"!\" = {}", str(123) + "!");
println("floor(3.7) = {}", floor(3.7));
println("ceil(3.2) = {}", ceil(3.2));
println("round(2.6) = {}", round(2.6));
println("float(3) = {}", float(3));
