// ==================================================================
// Exemple 112 — Une fonction est une valeur
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// On peut la ranger dans une variable et l'appeler.
// ------------------------------------------------------------------
// Sortie attendue :
//   42
// ==================================================================

func double(x: int) -> int {
    return x * 2;
}

let f = double;
println(f(21));
