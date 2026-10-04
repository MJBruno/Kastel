// ====================================================================
// Kastel — Affectations composées
// Notions : += -= *= /= %=
// Résultat attendu :
//   n = 15
//   n = 12
//   n = 24
//   n = 4
//   r = 6.0
// ====================================================================

let n = 10;

n += 5;
println("n = {}", n);
n -= 3;
println("n = {}", n);
n *= 2;
println("n = {}", n);
n %= 5;
println("n = {}", n);

let r = 24;
r /= 4;          // la division donne un float
println("r = {}", r);
