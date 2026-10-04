// ====================================================================
// Kastel — Référence ou copie ?
// Notions : une liste est une référence ; copy() duplique
// Résultat attendu :
//   4
//   3
// ====================================================================

let a = [1, 2, 3];
let b = a;          // même liste
let c = a.copy();   // copie indépendante

a.add(4);

println(b.size());
println(c.size());
