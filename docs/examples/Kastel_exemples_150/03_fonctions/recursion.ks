// ====================================================================
// Kastel — Récursivité
// Notions : factorielle et Fibonacci récursifs
// Résultat attendu :
//   factorielle(5) = 120
//   fib(10) = 55
// ====================================================================

func factorielle(n: int) -> int {
    if n <= 1 {
        return 1;
    }
    return n * factorielle(n - 1);
}

func fib(n: int) -> int {
    if n < 2 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

println("factorielle(5) = {}", factorielle(5));
println("fib(10) = {}", fib(10));
