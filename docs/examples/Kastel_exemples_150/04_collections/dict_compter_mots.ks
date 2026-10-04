// ====================================================================
// Kastel — Compter les mots
// Notions : split, get_or, dict comme compteur
// Résultat attendu :
//   le : 3
//   et : 2
//   mots différents : 5
// ====================================================================

let phrase = "le chat et le chien et le oiseau";
let compte = {};

for mot in phrase.split(" ") {
    compte[mot] = compte.get_or(mot, 0) + 1;
}

println("le : {}", compte["le"]);
println("et : {}", compte["et"]);
println("mots différents : {}", compte.size());
