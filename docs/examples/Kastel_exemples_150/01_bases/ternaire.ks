// ====================================================================
// Kastel — Expression conditionnelle
// Notions : cond ? a : b, ternaires imbriqués
// Résultat attendu :
//   Statut : majeur
//   Signe : positif
// ====================================================================

let age = 20;
let statut = age >= 18 ? "majeur" : "mineur";
println("Statut : {}", statut);

let n = 7;
let signe = n > 0 ? "positif" : n < 0 ? "negatif" : "nul";
println("Signe : {}", signe);
