// ====================================================================
// Kastel — Extraire une sous-chaîne
// Notions : slice(debut, fin), substring(debut, longueur), char_at
// Résultat attendu :
//   prog
//   gram
//   o
// ====================================================================

let mot = "programmation";

println(mot.slice(0, 4));
println(mot.substring(3, 4));
println(mot.char_at(2));
