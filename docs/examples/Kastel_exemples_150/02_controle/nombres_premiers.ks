// ====================================================================
// Kastel — Nombres premiers
// Notions : boucles imbriquées, break
// Résultat attendu :
//   2 3 5 7 11 13 17 19 23 29
// ====================================================================

let premiers = [];

for n in range(2, 30) {
    let est_premier = true;
    for d in range(2, n) {
        if d * d > n {
            break;
        }
        if n % d == 0 {
            est_premier = false;
            break;
        }
    }
    if est_premier {
        premiers.add(str(n));
    }
}

println(premiers.join(" "));
