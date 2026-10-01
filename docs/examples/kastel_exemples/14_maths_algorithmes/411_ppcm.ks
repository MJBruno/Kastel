// ==================================================================
// Exemple 411 — PPCM
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// ppcm(a, b) = a * b / pgcd(a, b).
// ------------------------------------------------------------------
// Sortie attendue :
//   12
//   42
// ==================================================================

func pgcd(a: int, b: int) -> int {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    return a;
}

func ppcm(a: int, b: int) -> int {
    return idiv(a * b, pgcd(a, b));
}

println(ppcm(4, 6));
println(ppcm(21, 6));
