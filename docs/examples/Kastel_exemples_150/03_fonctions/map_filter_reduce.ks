// ====================================================================
// Kastel — map, filter, reduce, any, all
// Notions : méthodes fonctionnelles des listes
// Résultat attendu :
//   [1, 4, 9, 16, 25]
//   [2, 4]
//   15
//   true
//   true
// ====================================================================

let nombres = [1, 2, 3, 4, 5];

let carres = nombres.map(x => x * x);
let pairs = nombres.filter(x => x % 2 == 0);
let somme = nombres.reduce((acc, x) => acc + x, 0);

println(carres);
println(pairs);
println(somme);
println(nombres.any(x => x > 4));
println(nombres.all(x => x > 0));
