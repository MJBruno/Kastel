// ====================================================================
// Kastel — break et continue
// Notions : continue saute un tour, break sort de la boucle
// Résultat attendu :
//   1
//   3
//   5
//   7
// ====================================================================

for i in range(1, 11) {
    if i % 2 == 0 {
        continue;
    }
    if i > 7 {
        break;
    }
    println(i);
}
