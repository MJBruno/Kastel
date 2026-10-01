// ==================================================================
// Exemple 304 — NotCallable
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Appeler une valeur qui n'est pas une fonction.
// ------------------------------------------------------------------
// Sortie attendue :
//   NotCallable
// ==================================================================

func appeler(x) {
    return x();
}

try {
    appeler(42);
} catch (e: Err) {
    println(e.kind);
}
