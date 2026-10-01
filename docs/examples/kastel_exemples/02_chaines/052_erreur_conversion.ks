// ==================================================================
// Exemple 052 — Conversion invalide
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// Un texte qui n'est pas un nombre lève TypeError.
// ------------------------------------------------------------------
// Sortie attendue :
//   erreur : TypeError
// ==================================================================

try {
    let n = "abc".to_int();
    println(n);
} catch (e: Err) {
    println("erreur : " + e.kind);
}
