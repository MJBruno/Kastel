// ====================================================================
// Kastel — Records
// Notions : { nom: valeur }, accès par point, keys, values
// Résultat attendu :
//   Ana
//   25
//   nom,age
// ====================================================================

let personne = { nom: "Ana", age: 25 };

println(personne.nom);
println(personne.age);
println(personne.keys().join(","));
