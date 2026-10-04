// ====================================================================
// Kastel — FizzBuzz
// Notions : modulo, conditions, join
// Résultat attendu :
//   1 2 Fizz 4 Buzz Fizz 7 8 Fizz Buzz 11 Fizz 13 14 FizzBuzz
// ====================================================================

let sortie = [];

for i in range(1, 16) {
    if i % 15 == 0 {
        sortie.add("FizzBuzz");
    } else if i % 3 == 0 {
        sortie.add("Fizz");
    } else if i % 5 == 0 {
        sortie.add("Buzz");
    } else {
        sortie.add(str(i));
    }
}

println(sortie.join(" "));
