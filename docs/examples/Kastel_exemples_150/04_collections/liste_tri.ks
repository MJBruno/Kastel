// ====================================================================
// Kastel — Trier une liste
// Notions : sort() en place (nombres ou chaînes homogènes)
// Résultat attendu :
//   [1, 2, 5, 9]
//   abricot, cerise, pomme
// ====================================================================

let nombres = [5, 2, 9, 1];
nombres.sort();
println(nombres);

let fruits = ["pomme", "abricot", "cerise"];
fruits.sort();
println(fruits.join(", "));
