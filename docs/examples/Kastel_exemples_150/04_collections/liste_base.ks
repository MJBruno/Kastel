// ====================================================================
// Kastel — Listes : les bases
// Notions : add, size, index, last, affectation par index
// Résultat attendu :
//   taille = 3
//   premier = pomme
//   dernier = cerise
//   pomme, kiwi, cerise
// ====================================================================

let fruits = ["pomme", "poire"];
fruits.add("cerise");

println("taille = {}", fruits.size());
println("premier = {}", fruits[0]);
println("dernier = {}", fruits.last());

fruits[1] = "kiwi";
println(fruits.join(", "));
