// ==================================================================
// Exemple 476 — Loterie : nombre de combinaisons
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : C(n, k) calculé avec la formule multiplicative (exacte en entiers).
// ------------------------------------------------------------------
// Sortie attendue :
//   13983816
//   10
// ==================================================================

func combinaisons(n: int, k: int) -> int {
    let r = 1;
    for i in range(1, k + 1) {
        r = idiv(r * (n - k + i), i);
    }
    return r;
}

println(combinaisons(49, 6));
println(combinaisons(5, 2));
