// ==================================================================
// Exemple 028 — wrapping_add : dépassement volontaire
// Catégorie : Bases du langage
// ------------------------------------------------------------------
// Les opérateurs lèvent une erreur au-delà de 64 bits ; wrapping_* recommence à l'autre bout.
// ------------------------------------------------------------------
// Sortie attendue :
//   -9223372036854775808
// ==================================================================

let max = 9223372036854775807;
println(wrapping_add(max, 1));   // bascule vers le minimum
