// ==================================================================
// Exemple 029 — Détecter un dépassement d'entier
// Catégorie : Bases du langage
// ------------------------------------------------------------------
// Un entier trop grand lève l'erreur IntegerOverflow, qu'on peut attraper.
// ------------------------------------------------------------------
// Sortie attendue :
//   IntegerOverflow
// ==================================================================

try {
    let x = 9223372036854775807;
    let y = x + 1;
    println(y);
} catch (e: Err) {
    println(e.kind);
}
