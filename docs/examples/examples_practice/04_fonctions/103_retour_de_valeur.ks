// ==================================================================
// Exemple 103 — Retourner une valeur
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// -> type indique le type de retour ; return renvoie la valeur.
// ------------------------------------------------------------------
// Sortie attendue :
//   49
//   16
// ==================================================================

func carre(x: int) -> int {
    return x * x;
}

println(carre(7));
println(carre(carre(2)));   // les appels s'imbriquent
