// ==================================================================
// Exemple 634 — Fibonacci par puissance de matrice
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : calculer F(80) avec seulement quelques multiplications de matrices 2 x 2.
// ------------------------------------------------------------------
// Sortie attendue :
//   12586269025
//   23416728348467685
// ==================================================================

func mul(a: List<List<int>>, b: List<List<int>>) -> List<List<int>> {
    return [
        [a[0][0] * b[0][0] + a[0][1] * b[1][0], a[0][0] * b[0][1] + a[0][1] * b[1][1]],
        [a[1][0] * b[0][0] + a[1][1] * b[1][0], a[1][0] * b[0][1] + a[1][1] * b[1][1]]
    ];
}

func fib(n: int) -> int {
    let res = [[1, 0], [0, 1]];
    let base = [[1, 1], [1, 0]];
    while n > 0 {
        if n % 2 == 1 { res = mul(res, base); }
        n = idiv(n, 2);
        if n > 0 { base = mul(base, base); }   // pas de carré inutile : évite un dépassement
    }
    return res[0][1];
}

println(fib(50));
println(fib(80));
