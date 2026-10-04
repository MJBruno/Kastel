// ====================================================================
// Kastel — Opérateurs arithmétiques
// Notions : + - * / % idiv pow ; la division donne toujours un float
// Résultat attendu :
//   17 + 5 = 22
//   17 - 5 = 12
//   17 * 5 = 85
//   17 / 5 = 3.4
//   17 % 5 = 2
//   idiv(17, 5) = 3
//   pow(2, 10) = 1024
// ====================================================================

let a = 17;
let b = 5;

println("{} + {} = {}", a, b, a + b);
println("{} - {} = {}", a, b, a - b);
println("{} * {} = {}", a, b, a * b);
println("{} / {} = {}", a, b, a / b);
println("{} % {} = {}", a, b, a % b);
println("idiv({}, {}) = {}", a, b, idiv(a, b));
println("pow(2, 10) = {}", pow(2, 10));
