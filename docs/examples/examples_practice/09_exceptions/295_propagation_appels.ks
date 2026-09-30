// ==================================================================
// Exemple 295 — Propagation à travers les fonctions
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Une erreur remonte la pile d'appels jusqu'au premier catch.
// ------------------------------------------------------------------
// Sortie attendue :
//   attrapé : boom
// ==================================================================

func niveau3() {
    throw "boom";
}
func niveau2() {
    niveau3();
    println("jamais affiché");
}
func niveau1() {
    niveau2();
}

try {
    niveau1();
} catch (e) {
    println("attrapé : " + e);
}
