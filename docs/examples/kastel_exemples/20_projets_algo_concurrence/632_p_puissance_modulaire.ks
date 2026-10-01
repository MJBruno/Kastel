// ==================================================================
// Exemple 632 — Exponentiation modulaire
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : a^b mod m en O(log b), sans jamais dépasser m².
// ------------------------------------------------------------------
// Sortie attendue :
//   24
//   9
// ==================================================================

func pow_mod(base: int, exp: int, m: int) -> int {
    let res = 1;
    base = base % m;
    while exp > 0 {
        if exp % 2 == 1 {
            res = res * base % m;
        }
        base = base * base % m;
        exp = idiv(exp, 2);
    }
    return res;
}

println(pow_mod(2, 10, 1000));
println(pow_mod(3, 200, 13));
