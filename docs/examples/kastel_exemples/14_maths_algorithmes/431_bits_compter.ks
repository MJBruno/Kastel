// ==================================================================
// Exemple 431 — Compter les bits à 1
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Décaler et masquer avec >> et &.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
//   8
//   1
// ==================================================================

func popcount(n: int) -> int {
    let c = 0;
    while n > 0 {
        c += n & 1;
        n = n >> 1;
    }
    return c;
}

println(popcount(7));     // 111
println(popcount(255));   // 8 bits
println(popcount(1024));
