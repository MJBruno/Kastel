// examples/std_math_demo.ks
//
// Petit tour des fonctionnalités de std.math.
// Lancer avec : kastel examples/std_math_demo.ks

// -- Import qualifié (fonctions) vs import direct (classe) -----------------
//
// `import std.math;` charge le MODULE lui-même : les fonctions s'y
// appellent par `math.xxx(...)`, y compris les natives ré-exportées
// (sqrt, pow, sin, ...).
import std.math;

println(math.sqrt(2.0));           // ~1.414...  (native ré-exportée)
println(math.gcd(48, 18));         // 6          (fonction pure Kastel)

// `import std.math.Complexe;` importe directement l'EXPORT "Complexe"
// (une classe) du module std.math : pas de qualification, comme pour
// n'importe quelle classe importée.
import std.math.Complexe;

let z1: Complexe = new Complexe(3, 4);
let z2 = new Complexe(1, 2);

println(z1.to_string());           // 3+4i
println(z1.add(z2).to_string());   // 4+6i
println(z1.multiply(z2).to_string()); // -5+10i
println(z1.magnitude());           // 5.0


// -- Constantes et angles -------------------------------------------------
println(math.PI);
println(math.TAU);
println(math.to_radians(180));          // ~PI
println(math.to_degrees(math.PI));      // 180.0

// -- Trigonométrie (natives ré-exportées par std.math) ---------------------
println(math.sin(0));                   // 0.0
println(math.atan2(1, 1));              // ~0.785398... (PI / 4)

// -- Hyperboliques ----------------------------------------------------------
println(math.sinh(0));                  // 0.0
println(math.cosh(0));                  // 1.0
println(math.tanh(0));                  // 0.0

// -- Racines et distances ----------------------------------------------------
println(math.cbrt(27));                 // ~3.0
println(math.cbrt(-8));                 // ~-2.0
println(math.hypot(3, 4));              // 5.0

// -- Arrondi et signe --------------------------------------------------------
println(math.sign(-42));                // -1
println(math.sign(0));                  // 0
println(math.trunc(-1.7));              // -1
println(math.clamp(15, 0, 10));         // 10

// -- Interpolation -------------------------------------------------------------
println(math.lerp(0, 100, 0.25));       // 25.0
println(math.inverse_lerp(0, 100, 25)); // 0.25
println(math.map_range(5, 0, 10, 0, 100)); // 50.0

// -- Théorie des nombres --------------------------------------------------------
println(math.lcm(4, 6));                // 12
println(math.is_prime(17));             // true
println(math.is_prime(18));             // false
println(math.factorial(5));             // 120
println(math.fibonacci(10));            // 55

// -- Combinatoire -----------------------------------------------------------------
println(math.permutations(5, 2));       // 20
println(math.combinations(5, 2));       // 10

// -- Statistiques -----------------------------------------------------------------
let data: Array<int> = [4, 8, 6, 5, 3, 7, 8];

println(math.sum(data));                // 41
println(math.average(data));            // ~5.857...
println(math.median(data));             // 6
println(math.variance(data));
println(math.std_dev(data));
println(math.mode(data));               // 8
println(math.min_of(data));             // 3
println(math.max_of(data));             // 8

// -- Aléatoire ------------------------------------------------------------------------
println(math.choice(data));             // un élément au hasard de `data`
println(math.shuffle(data));            // `data` mélangé (copie, `data` inchangé)
