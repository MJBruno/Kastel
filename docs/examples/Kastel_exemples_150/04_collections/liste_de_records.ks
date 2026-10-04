// ====================================================================
// Kastel — Liste de records
// Notions : filter + map sur des records
// Résultat attendu :
//   Bruno, Chloé
// ====================================================================

let personnes = [
    { nom: "Ana", age: 17 },
    { nom: "Bruno", age: 25 },
    { nom: "Chloé", age: 31 }
];

let majeurs = personnes.filter(p => p.age >= 18).map(p => p.nom);
println(majeurs.join(", "));
