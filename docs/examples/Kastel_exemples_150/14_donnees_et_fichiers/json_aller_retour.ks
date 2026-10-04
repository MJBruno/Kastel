// ====================================================================
// Kastel — JSON
// Notions : json_decode, json_encode
// Résultat attendu :
//   Ana
//   25
//   3
// ====================================================================

let donnees = json_decode("{\"nom\": \"Ana\", \"age\": 25}");
println(donnees["nom"]);
println(donnees["age"]);

let texte = json_encode([1, 2, 3]);
println(json_decode(texte).size());
