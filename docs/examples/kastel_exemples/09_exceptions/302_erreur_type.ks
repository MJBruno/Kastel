// ==================================================================
// Exemple 302 — TypeError
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Additionner un nombre et un texte est une erreur de type.
// ------------------------------------------------------------------
// Sortie attendue :
//   TypeError
// ==================================================================

func addition(a, b) {
    return a + b;
}

try {
    println(addition(1, "2"));
} catch (e: Err) {
    println(e.kind);
}
