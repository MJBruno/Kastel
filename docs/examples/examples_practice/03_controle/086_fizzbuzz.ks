// ==================================================================
// Exemple 086 — FizzBuzz
// Catégorie : Structures de contrôle
// ------------------------------------------------------------------
// Le classique : multiples de 3, 5 et des deux.
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   2
//   Fizz
//   4
//   Buzz
//   Fizz
//   7
//   8
//   Fizz
//   Buzz
//   11
//   Fizz
//   13
//   14
//   FizzBuzz
// ==================================================================

for i in range(1, 16) {
    if i % 15 == 0 {
        println("FizzBuzz");
    } else if i % 3 == 0 {
        println("Fizz");
    } else if i % 5 == 0 {
        println("Buzz");
    } else {
        println(i);
    }
}
