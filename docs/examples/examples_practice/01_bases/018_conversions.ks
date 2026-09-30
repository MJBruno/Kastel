// ==================================================================
// Exemple 018 — Convertir avec int(), float(), str(), bool()
// Catégorie : Bases du langage
// ------------------------------------------------------------------
// Les fonctions de conversion acceptent aussi du texte.
// ------------------------------------------------------------------
// Sortie attendue :
//   43
//   2.5
//   42!
//   3
//   false
//   true
// ==================================================================

println(int("42") + 1);    // 43
println(float("2.5"));     // 2.5
println(str(42) + "!");    // 42!
println(int(3.9));         // 3 (tronqué)
println(bool(0));          // false
println(bool("a"));        // true
