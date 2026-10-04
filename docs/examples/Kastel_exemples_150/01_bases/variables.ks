// ====================================================================
// Kastel — Variables et constantes
// Notions : let, const, annotations de type, réaffectation
// Résultat attendu :
//   Alice a 30 ans
//   Taille : 1.68 m
//   Actif : true
//   PI = 3.14159
//   L'an prochain : 31 ans
// ====================================================================

let nom = "Alice";
let age: int = 30;
let taille: float = 1.68;
let actif: bool = true;
const PI = 3.14159;

println("{} a {} ans", nom, age);
println("Taille : {} m", taille);
println("Actif : {}", actif);
println("PI = {}", PI);

age = age + 1;
println("L'an prochain : {} ans", age);
