// ====================================================================
// Kastel — Boucle for et range
// Notions : range(n), range(a, b)
// Résultat attendu :
//   0 1 2 3 4
//   1 2 3 4 5
// ====================================================================

let a = [];
for i in range(5) {
    a.add(str(i));
}
println(a.join(" "));

let b = [];
for i in range(1, 6) {
    b.add(str(i));
}
println(b.join(" "));
