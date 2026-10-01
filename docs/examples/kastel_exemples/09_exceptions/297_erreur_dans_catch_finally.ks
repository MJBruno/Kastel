// ==================================================================
// Exemple 297 — Erreur dans catch : finally exécuté
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Même si le catch lève à son tour, le finally a lieu.
// ------------------------------------------------------------------
// Sortie attendue :
//   finally
//   reçu : b
// ==================================================================

try {
    try {
        throw "a";
    } catch (e) {
        throw "b";
    } finally {
        println("finally");
    }
} catch (e) {
    println("reçu : " + e);
}
