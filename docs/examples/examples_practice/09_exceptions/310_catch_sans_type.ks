// ==================================================================
// Exemple 310 — catch (e) attrape aussi les erreurs internes
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Sans annotation, le catch reçoit toute valeur lancée ou erreur runtime.
// ------------------------------------------------------------------
// Sortie attendue :
//   attrapé quelque chose
// ==================================================================

try {
    let v = [];
    v[3] = 1;
} catch (e) {
    println("attrapé quelque chose");
}
