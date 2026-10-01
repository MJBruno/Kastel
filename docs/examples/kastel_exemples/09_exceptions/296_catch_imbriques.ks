// ==================================================================
// Exemple 296 — try imbriqués
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Le catch le plus proche gère l'erreur ; il peut la relancer.
// ------------------------------------------------------------------
// Sortie attendue :
//   interne : origine
//   externe : relancée
// ==================================================================

try {
    try {
        throw "origine";
    } catch (e) {
        println("interne : " + e);
        throw "relancée";
    }
} catch (e2) {
    println("externe : " + e2);
}
