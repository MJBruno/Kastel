// ==================================================================
// Exemple 445 — JSON invalide
// Catégorie : Fichiers, JSON, système
// ------------------------------------------------------------------
// Un texte mal formé lève une erreur.
// ------------------------------------------------------------------
// Sortie attendue :
//   JSON invalide
// ==================================================================

try {
    json_decode("{ pas du json");
} catch (e) {
    println("JSON invalide");
}
