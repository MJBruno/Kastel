// ====================================================================
// Kastel — Range avec un pas
// Notions : range(debut, fin, pas), pas négatif
// Résultat attendu :
//   0 3 6 9
//   5 4 3 2 1
// ====================================================================

let montee = [];
for i in range(0, 10, 3) {
    montee.add(str(i));
}
println(montee.join(" "));

let descente = [];
for i in range(5, 0, -1) {
    descente.add(str(i));
}
println(descente.join(" "));
