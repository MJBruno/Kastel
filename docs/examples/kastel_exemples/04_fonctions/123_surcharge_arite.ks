// ==================================================================
// Exemple 123 — Surcharge selon le nombre d'arguments
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Le nombre de paramètres suffit aussi à choisir la version.
// ------------------------------------------------------------------
// Sortie attendue :
//   16
//   20
// ==================================================================

func aire(cote: int) -> int {
    return cote * cote;
}

func aire(largeur: int, hauteur: int) -> int {
    return largeur * hauteur;
}

println(aire(4));
println(aire(4, 5));
