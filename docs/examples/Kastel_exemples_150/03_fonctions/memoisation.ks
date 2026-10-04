// ====================================================================
// Kastel — Mémoïsation
// Notions : dict comme cache, index assignment
// Résultat attendu :
//   12586269025
// ====================================================================

let memo = {};

func fib(n) {
    if n < 2 {
        return n;
    }

    let cle = str(n);
    if memo.contains(cle) {
        return memo[cle];
    }

    let valeur = fib(n - 1) + fib(n - 2);
    memo[cle] = valeur;
    return valeur;
}

println(fib(50));
