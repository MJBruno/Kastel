// ==================================================================
// Exemple 363 — Fibonacci asynchrone
// Catégorie : async / await
// ------------------------------------------------------------------
// Chaque appel récursif est une tâche.
// ------------------------------------------------------------------
// Sortie attendue :
//   55
// ==================================================================

async func fib(n: int) -> int {
    if n < 2 {
        return n;
    }
    let a = fib(n - 1);
    let b = fib(n - 2);
    return (await a) + (await b);
}

println(await fib(10));
