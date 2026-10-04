// ====================================================================
// Kastel — Conjecture de Collatz
// Notions : while, if / else
// Résultat attendu :
//   6 -> 1 en 8 étapes
// ====================================================================

let depart = 6;
let n = depart;
let etapes = 0;

while n != 1 {
    if n % 2 == 0 {
        n = idiv(n, 2);
    } else {
        n = 3 * n + 1;
    }
    etapes += 1;
}

println("{} -> 1 en {} étapes", depart, etapes);
