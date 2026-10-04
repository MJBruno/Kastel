// ====================================================================
// Kastel — Boucles imbriquées
// Notions : table de multiplication
// Résultat attendu :
//   1 2 3
//   2 4 6
//   3 6 9
// ====================================================================

for i in range(1, 4) {
    let ligne = [];
    for j in range(1, 4) {
        ligne.add(str(i * j));
    }
    println(ligne.join(" "));
}
