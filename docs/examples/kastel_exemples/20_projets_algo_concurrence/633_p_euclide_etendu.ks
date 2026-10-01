// ==================================================================
// Exemple 633 — Algorithme d'Euclide étendu
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : trouver (pgcd, x, y) avec a*x + b*y = pgcd.
// ------------------------------------------------------------------
// Sortie attendue :
//   (2, -9, 47)
//   2
// ==================================================================

func egcd(a: int, b: int) -> Tuple<int, int, int> {
    if b == 0 {
        return (a, 1, 0);
    }
    let r = egcd(b, a % b);
    return (r[0], r[2], r[1] - idiv(a, b) * r[2]);
}

let r = egcd(240, 46);
println(r);
println(240 * r[1] + 46 * r[2]);
