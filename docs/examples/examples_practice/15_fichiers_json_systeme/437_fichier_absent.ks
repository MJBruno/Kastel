// ==================================================================
// Exemple 437 — Lire un fichier absent
// Catégorie : Fichiers, JSON, système
// ------------------------------------------------------------------
// Une erreur est levée : on l'attrape.
// ------------------------------------------------------------------
// Sortie attendue :
//   lecture impossible
// ==================================================================

try {
    file_read("introuvable_kastel.txt");
} catch (e) {
    println("lecture impossible");
}
