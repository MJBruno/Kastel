// ==================================================================
// Exemple 640 — PPCM d'une liste
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : le plus petit nombre divisible par 1, 2, ..., 10.
// ------------------------------------------------------------------
// Sortie attendue :
//   2520
// ==================================================================

func pgcd(a: int, b: int) -> int {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    return a;
}

let p = 1;
for n in range(1, 11) {
    p = idiv(p * n, pgcd(p, n));
}
println(p);
