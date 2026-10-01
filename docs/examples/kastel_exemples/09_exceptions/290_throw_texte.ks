// ==================================================================
// Exemple 290 — Lancer un texte avec throw
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// throw peut lancer n'importe quelle valeur.
// ------------------------------------------------------------------
// Sortie attendue :
//   quelque chose s'est mal passé
// ==================================================================

try {
    throw "quelque chose s'est mal passé";
} catch (e) {
    println(e);
}
