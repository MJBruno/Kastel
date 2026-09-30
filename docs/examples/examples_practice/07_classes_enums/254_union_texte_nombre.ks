// ==================================================================
// Exemple 254 — Union de types différents
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Un paramètre peut accepter un texte ou un entier.
// ------------------------------------------------------------------
// Sortie attendue :
//   valeur : 42
//   valeur : kastel
// ==================================================================

func afficher(x: int | str) {
    println("valeur : {}", x);
}

afficher(42);
afficher("kastel");
