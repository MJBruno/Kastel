// ==================================================================
// Exemple 056 — Tester un palindrome
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// Comparer une chaîne à son inverse.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   true
//   false
// ==================================================================

func est_palindrome(s: str) -> bool {
    let propre = s.lower().replace_all(" ", "");
    return propre == propre.reverse();
}

println(est_palindrome("radar"));
println(est_palindrome("Esope reste ici et se repose"));   // espaces et casse ignorés
println(est_palindrome("kastel"));
