// ====================================================================
// Kastel — Alias de type générique
// Notions : type Paire<A, B> = { ... }, instanciation Paire<int, str>
// Résultat attendu :
//   a = 1, b = x
//   a = 7, b = kastel
// ====================================================================

type Paire<A, B> = { gauche: A, droite: B };

let p: Paire<int, str> = { gauche: 1, droite: "x" };
println("a = {}, b = {}", p.gauche, p.droite);

let q: Paire<int, str> = { gauche: 7, droite: "kastel" };
println("a = {}, b = {}", q.gauche, q.droite);
