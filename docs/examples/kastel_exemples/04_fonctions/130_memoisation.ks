// ==================================================================
// Exemple 130 — Mémoïsation avec un dict
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Garder les résultats déjà calculés pour éviter de refaire le travail.
// ------------------------------------------------------------------
// Sortie attendue :
//   12586269025
// ==================================================================

let cache = dict();

func fib(n: int) -> int {
    if n < 2 {
        return n;
    }
    if cache.contains(n) {
        return cache[n];
    }
    let r = fib(n - 1) + fib(n - 2);
    cache[n] = r;
    return r;
}

println(fib(50));
