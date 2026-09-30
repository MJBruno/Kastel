// ==================================================================
// Exemple 125 — Fonction générique
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// <T> laisse la fonction accepter n'importe quel type en gardant la cohérence.
// ------------------------------------------------------------------
// Sortie attendue :
//   42
//   kastel
//   2.5
// ==================================================================

func identite<T>(valeur: T) -> T {
    return valeur;
}

println(identite(42));
println(identite("kastel"));
println(identite<float>(2.5));
