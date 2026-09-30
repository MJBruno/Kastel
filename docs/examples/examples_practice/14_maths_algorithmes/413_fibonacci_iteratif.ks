// ==================================================================
// Exemple 413 — Fibonacci itératif
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Deux variables qui avancent : rapide et sans récursion.
// ------------------------------------------------------------------
// Sortie attendue :
//   55
//   12586269025
// ==================================================================

func fib(n: int) -> int {
    let a = 0;
    let b = 1;
    for i in range(n) {
        let tmp = a + b;
        a = b;
        b = tmp;
    }
    return a;
}

println(fib(10));
println(fib(50));
