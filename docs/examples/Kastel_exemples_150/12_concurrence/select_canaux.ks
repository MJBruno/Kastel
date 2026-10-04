// ====================================================================
// Kastel — Attendre plusieurs canaux
// Notions : select([...]) renvoie (index, valeur, fermé) ; select avec timeout
// Résultat attendu :
//   canal 1 -> 22
//   timeout : -1
// ====================================================================

let a = channel();
let b = channel();
b.send(22);

let r = select([a, b]);
println("canal {} -> {}", r[0], r[1]);

let vide = channel();
let t = select([vide], 0);       // timeout de 0 ms
println("timeout : {}", t[0]);
