// ==================================================================
// Exemple 119 — Fabrique de fonctions
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Une fonction qui renvoie une fonction configurée.
// ------------------------------------------------------------------
// Sortie attendue :
//   6
//   101
// ==================================================================

func additionneur(k: int) {
    return x => x + k;
}

let plus5 = additionneur(5);
let plus100 = additionneur(100);
println(plus5(1));
println(plus100(1));
