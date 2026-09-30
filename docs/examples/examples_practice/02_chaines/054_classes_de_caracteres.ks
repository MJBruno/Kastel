// ==================================================================
// Exemple 054 — is_digit, is_alpha, is_alphanumeric
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// Chaque test exige que TOUS les caractères correspondent (et que le texte ne soit pas vide).
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
//   true
//   false
//   true
// ==================================================================

println("123".is_digit());        // true
println("12a".is_digit());        // false
println("abc".is_alpha());        // true
println("abc1".is_alpha());       // false
println("abc1".is_alphanumeric()); // true
