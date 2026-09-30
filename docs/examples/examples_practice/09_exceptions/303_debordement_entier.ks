// ==================================================================
// Exemple 303 — IntegerOverflow
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Les entiers 64 bits ne débordent jamais en silence.
// ------------------------------------------------------------------
// Sortie attendue :
//   IntegerOverflow
// ==================================================================

func fois_deux(n: int) -> int {
    return n * 2;
}

try {
    println(fois_deux(9223372036854775807));
} catch (e: Err) {
    println(e.kind);
}
