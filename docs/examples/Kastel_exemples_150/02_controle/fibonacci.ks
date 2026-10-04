// ====================================================================
// Kastel — Suite de Fibonacci
// Notions : deux variables, boucle
// Résultat attendu :
//   0 1 1 2 3 5 8 13 21 34
// ====================================================================

let a = 0;
let b = 1;
let termes = [];

for i in range(10) {
    termes.add(str(a));
    let suivant = a + b;
    a = b;
    b = suivant;
}

println(termes.join(" "));
