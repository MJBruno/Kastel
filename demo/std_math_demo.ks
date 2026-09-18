// examples/std_math_demo.ks
//
// Petit tour des fonctionnalités de std.math.
// Lancer avec : kastel examples/std_math_demo.ks

import {
    PI, TAU, to_radians, 
    to_degrees, sinh, cosh, 
    tanh, cbrt, hypot, sign,
    trunc, clamp, lerp, inverse_lerp, 
    map_range, gcd, lcm, is_prime, 
    factorial, fibonacci, permutations, 
    combinations, sum, average, median, 
    variance, std_dev, mode, min_of, max_of,
    choice, shuffle
} from std.math ;

// -- Constantes et angles -------------------------------------------------
println(PI);
println(TAU);
println(to_radians(180));          // ~PI
println(to_degrees(PI));           // 180.0

// -- Trigonométrie (natives globales, sans import) -------------------------
println(sin(0));                   // 0.0
println(atan2(1, 1));              // ~0.785398... (PI / 4)

// -- Hyperboliques ----------------------------------------------------------
println(sinh(0));                  // 0.0
println(cosh(0));                  // 1.0
println(tanh(0));                  // 0.0

// -- Racines et distances ----------------------------------------------------
println(cbrt(27));                 // ~3.0
println(cbrt(-8));                 // ~-2.0
println(hypot(3, 4));               // 5.0

// -- Arrondi et signe --------------------------------------------------------
println(sign(-42));                // -1
println(sign(0));                  // 0
println(trunc(-1.7));              // -1
println(clamp(15, 0, 10));         // 10

// -- Interpolation -------------------------------------------------------------
println(lerp(0, 100, 0.25));       // 25.0
println(inverse_lerp(0, 100, 25)); // 0.25
println(map_range(5, 0, 10, 0, 100)); // 50.0

// -- Théorie des nombres --------------------------------------------------------
println(gcd(48, 18));              // 6
println(lcm(4, 6));                // 12
println(is_prime(17));             // true
println(is_prime(18));             // false
println(factorial(5));             // 120
println(fibonacci(10));            // 55

// -- Combinatoire -----------------------------------------------------------------
println(permutations(5, 2));       // 20
println(combinations(5, 2));       // 10

// -- Statistiques -----------------------------------------------------------------
let data = [4, 8, 6, 5, 3, 7, 8];

println(sum(data));                // 41
println(average(data));            // ~5.857...
println(median(data));             // 6
println(variance(data));
println(std_dev(data));
println(mode(data));               // 8
println(min_of(data));             // 3
println(max_of(data));             // 8

// -- Aléatoire ------------------------------------------------------------------------
println(choice(data));             // un élément au hasard de `data`
println(shuffle(data));            // `data` mélangé (copie, `data` inchangé)
