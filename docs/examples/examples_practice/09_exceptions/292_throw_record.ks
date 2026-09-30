// ==================================================================
// Exemple 292 — Lancer un record
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Un record porte plusieurs informations.
// ------------------------------------------------------------------
// Sortie attendue :
//   500 - erreur serveur
// ==================================================================

try {
    throw { code: 500, texte: "erreur serveur" };
} catch (e) {
    println("{} - {}", e.code, e.texte);
}
