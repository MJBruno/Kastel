// ==================================================================
// Exemple 294 — try / finally sans catch
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// L'erreur continue de remonter, mais finally s'exécute d'abord.
// ------------------------------------------------------------------
// Sortie attendue :
//   finally interne
//   attrapé plus haut : erreur interne
// ==================================================================

try {
    try {
        throw "erreur interne";
    } finally {
        println("finally interne");
    }
} catch (e) {
    println("attrapé plus haut : " + e);
}
