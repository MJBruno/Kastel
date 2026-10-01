// ==================================================================
// Exemple 034 — Valeurs « vraies » et « fausses »
// Catégorie : Bases du langage
// ------------------------------------------------------------------
// bool() montre comment chaque valeur est interprétée dans une condition.
// ------------------------------------------------------------------
// Sortie attendue :
//   false
//   true
//   false
//   true
//   false
// ==================================================================

println(bool(0));      // false
println(bool(1));      // true
println(bool(""));     // false
println(bool("a"));    // true
println(bool(None));   // false
