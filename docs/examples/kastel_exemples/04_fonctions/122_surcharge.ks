// ==================================================================
// Exemple 122 — Surcharge de fonctions
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Deux fonctions du même nom, distinguées par le type des paramètres.
// ------------------------------------------------------------------
// Sortie attendue :
//   entier 42
//   texte kastel
// ==================================================================

func decrire(x: int) -> str {
    return "entier " + str(x);
}

func decrire(x: str) -> str {
    return "texte " + x;
}

println(decrire(42));
println(decrire("kastel"));
