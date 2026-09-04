let a = 7;
let b = 2;

println(type(a));    // "int"
println(a + b);       // 9   (int + int -> int)
println(a / b);        // 3.5 (division toujours "vraie", façon Python 3)
println(a % b);         // 1   (int)

let c = 2.5;
println(type(c));        // "float"
println(a + c);           // 9.5 (int + float -> float, promotion)

println(5 == 5.0);         // true (comparables entre types)

let x = 5.0;
println(x);                 // "5.0" (jamais "5" — distingue visuellement du int)

println(3 & 5);              // 1 (bitwise exige un vrai Integer désormais)
// println(3.0 & 5);             // 🚨 Erreur de type — plus de troncature implicite

for i in range(3) {
    println(type(i));          // "int", trois fois
}