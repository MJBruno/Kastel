// ====================================================================
// Kastel — Factorielle itérative
// Notions : boucle for, produit
// Résultat attendu :
//   5! = 120
//   10! = 3628800
//   20! = 2432902008176640000
// ====================================================================

func factorielle(n) {
    let resultat = 1;
    for i in range(2, n + 1) {
        resultat = resultat * i;
    }
    return resultat;
}

println("5! = {}", factorielle(5));
println("10! = {}", factorielle(10));
println("20! = {}", factorielle(20));
