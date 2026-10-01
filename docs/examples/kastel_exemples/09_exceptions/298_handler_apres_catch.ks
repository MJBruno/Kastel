// ==================================================================
// Exemple 298 — Un catch n'intercepte plus après avoir fini
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Une fois le catch terminé, une erreur suivante remonte bien au catch externe.
// ------------------------------------------------------------------
// Sortie attendue :
//   interne
//   externe : seconde
// ==================================================================

try {
    try {
        throw "premiere";
    } catch (e) {
        println("interne");
    }
    throw "seconde";
} catch (e) {
    println("externe : " + e);
}
