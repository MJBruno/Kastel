// ==================================================================
// Exemple 300 — return dans finally
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Un return dans finally remplace celui du try.
// ------------------------------------------------------------------
// Sortie attendue :
//   2
// ==================================================================

func g() -> int {
    try {
        return 1;
    } finally {
        return 2;
    }
}

println(g());
