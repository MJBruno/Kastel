// ====================================================================
// Kastel — Somme des chiffres
// Notions : while, % et idiv
// Résultat attendu :
//   Somme des chiffres de 12345 : 15
// ====================================================================

let nombre = 12345;
let n = nombre;
let somme = 0;

while n > 0 {
    somme += n % 10;
    n = idiv(n, 10);
}

println("Somme des chiffres de {} : {}", nombre, somme);
