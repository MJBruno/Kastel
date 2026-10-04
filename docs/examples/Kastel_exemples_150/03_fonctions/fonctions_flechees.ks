// ====================================================================
// Kastel — Fonctions fléchées
// Notions : x => ..., (a, b) => ..., () => ...
// Résultat attendu :
//   10
//   7
//   true
//   salut
// ====================================================================

let double = x => x * 2;
let additionner = (a, b) => a + b;
let est_pair = n => n % 2 == 0;
let dire = () => "salut";

println(double(5));
println(additionner(3, 4));
println(est_pair(8));
println(dire());
