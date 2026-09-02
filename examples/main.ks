println(int(3.9));        // 3   (tronqué vers zéro, pas arrondi)
println(int(-3.9));       // -3  (comme Python : truncation, pas floor)
println(int("42"));       // 42
println(int(true));       // 1

println(float("3.14"));   // 3.14
println(float(false));    // 0
let a= [1, 2, 3];
println(str(42));         // "42"
println(str(a));            // "[1, 2, 3]"
println(str({ a: 1 }));   // "{a: 1}"

println(bool(0));         // false
println(bool(""));        // false
println(bool([]));        // true   (tableau vide reste "truthy" en Kastel)
println(bool(nil));       // false