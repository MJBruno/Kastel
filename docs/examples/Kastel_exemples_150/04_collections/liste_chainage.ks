// ====================================================================
// Kastel — Chaîner filter, map et reduce
// Notions : somme des carrés des nombres pairs
// Résultat attendu :
//   56
// ====================================================================

let resultat = [1, 2, 3, 4, 5, 6]
    .filter(x => x % 2 == 0)
    .map(x => x * x)
    .reduce((acc, x) => acc + x, 0);

println(resultat);
