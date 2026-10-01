// ==================================================================
// Exemple 133 — Mauvais nombre d'arguments
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Appeler une fonction avec trop ou pas assez d'arguments lève une erreur.
// ------------------------------------------------------------------
// Sortie attendue :
//   WrongArgumentCount
// ==================================================================

let f = (a, b) => a + b;
try {
    f(1);
} catch (e: Err) {
    println(e.kind);
}
