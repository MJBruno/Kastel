// ==================================================================
// Exemple 105 — Paramètres sans annotation
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Les types sont facultatifs : la fonction accepte alors n'importe quelle valeur.
// ------------------------------------------------------------------
// Sortie attendue :
//   42
//   abab
// ==================================================================

func doubler(x) {
    return x + x;
}

println(doubler(21));
println(doubler("ab"));
