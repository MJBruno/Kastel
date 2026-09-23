// test/std/test_math.ks
//
// À exécuter avec : kastel test/std/test_math.ks
// (ou agrégé depuis run_all.ks)

import std.math;
import std.math.Complexe; // une classe s'importe SANS qualifier par le module : new Complexe(...), pas new math.Complexe(...)
from std.testing import assert_eq, assert_true, assert_false, run_tests;

func close(a, b) {
    return abs(a - b) < 0.00000001;
}

func test_constants() {
    assert_true(close(math.PI, 3.141592653589793), "PI");
    assert_true(close(math.TAU, 2 * math.PI), "TAU == 2*PI");
}

func test_reexported_natives_qualified_access() {
    // math.sqrt doit fonctionner exactement comme le sqrt global — c'est
    // tout l'intérêt du `export const sqrt = sqrt;`.
    assert_eq(math.sqrt(9), sqrt(9), "math.sqrt == sqrt global");
    assert_eq(math.abs(-3), 3, "math.abs");
}

func test_angles() {
    assert_true(close(math.to_radians(180), math.PI), "180° -> PI radians");
    assert_true(close(math.to_degrees(math.PI), 180), "PI radians -> 180°");
}

func test_hyperbolic() {
    assert_true(close(math.sinh(0), 0), "sinh(0) == 0");
    assert_true(close(math.cosh(0), 1), "cosh(0) == 1");
    assert_true(close(math.tanh(0), 0), "tanh(0) == 0");
}

func test_roots_and_distance() {
    assert_true(close(math.cbrt(27), 3), "cbrt(27) == 3");
    assert_true(close(math.cbrt(-8), -2), "cbrt(-8) == -2 (négatif géré)");
    assert_true(close(math.hypot(3, 4), 5), "hypot(3, 4) == 5");
}

func test_rounding_and_sign() {
    assert_eq(math.sign(5), 1, "sign(5)");
    assert_eq(math.sign(-5), -1, "sign(-5)");
    assert_eq(math.sign(0), 0, "sign(0)");
    assert_true(close(math.trunc(-1.5), -1), "trunc(-1.5) == -1 (vers zéro)");
    assert_true(close(math.trunc(1.5), 1), "trunc(1.5) == 1");
}

func test_clamp() {
    assert_eq(math.clamp(5, 0, 10), 5, "clamp dans l'intervalle");
    assert_eq(math.clamp(-5, 0, 10), 0, "clamp sous le minimum");
    assert_eq(math.clamp(15, 0, 10), 10, "clamp au-dessus du maximum");
}

func test_interpolation() {
    assert_true(close(math.lerp(0, 10, 0.5), 5), "lerp(0, 10, 0.5) == 5");
    assert_true(close(math.inverse_lerp(0, 10, 5), 0.5), "inverse_lerp inverse de lerp");
    assert_true(close(math.map_range(5, 0, 10, 0, 100), 50), "map_range");
}

func test_number_theory() {
    assert_eq(math.gcd(48, 18), 6, "gcd(48, 18)");
    assert_eq(math.gcd(-48, 18), 6, "gcd gère les négatifs via abs()");
    assert_eq(math.lcm(4, 6), 12, "lcm(4, 6)");
    assert_eq(math.lcm(0, 5), 0, "lcm avec zéro");
    assert_true(math.is_prime(13), "13 est premier");
    assert_false(math.is_prime(1), "1 n'est pas premier");
    assert_false(math.is_prime(15), "15 n'est pas premier");
}

func test_factorial_and_fibonacci() {
    assert_eq(math.factorial(0), 1, "0! == 1");
    assert_eq(math.factorial(5), 120, "5! == 120");
    assert_eq(math.fibonacci(0), 0, "fib(0)");
    assert_eq(math.fibonacci(1), 1, "fib(1)");
    assert_eq(math.fibonacci(10), 55, "fib(10)");
}

func test_factorial_rejects_negative() {
    try {
        math.factorial(-1);
        throw "factorial(-1) aurait dû lever une erreur";
    } catch error {
        // attendu
    }
}

func test_combinatorics() {
    assert_eq(math.permutations(5, 2), 20, "P(5,2) == 20");
    assert_eq(math.combinations(5, 2), 10, "C(5,2) == 10");
    assert_eq(math.combinations(5, 0), 1, "C(5,0) == 1");
    assert_eq(math.permutations(5, 6), 0, "P(5,6) == 0 (r > n)");
}

func test_aggregates() {
    let data = [2, 4, 4, 4, 5, 5, 7, 9];

    assert_eq(math.sum(data), 40, "sum");
    assert_true(close(math.average(data), 5), "average");
    assert_true(close(math.median(data), 4.5), "median (n pair)");
    assert_true(close(math.median([1, 2, 3]), 2), "median (n impair)");
    assert_true(close(math.variance(data), 4), "variance (population)");
    assert_true(close(math.std_dev(data), 2), "std_dev");
    assert_eq(math.mode(data), 4, "mode");
    assert_eq(math.min_of(data), 2, "min_of");
    assert_eq(math.max_of(data), 9, "max_of");
}

func test_median_empty_throws() {
    try {
        math.median([]);
        throw "median([]) aurait dû lever une erreur";
    } catch error {
        // attendu
    }
}

func test_random_within_bounds() {
    let i = 0;
    while i < 50 {
        let x = math.random_range(1.0, 2.0);
        assert_true(x >= 1.0 && x < 2.0, "random_range dans les bornes");

        let picked = math.choice([1, 2, 3]);
        assert_true(picked == 1 || picked == 2 || picked == 3, "choice renvoie un élément du tableau");

        i = i + 1;
    }
}

func test_shuffle_is_a_permutation_and_does_not_mutate() {
    let original = [1, 2, 3, 4, 5];
    let shuffled = math.shuffle(original);

    assert_eq(original.size(), 5, "shuffle ne modifie pas l'original");
    assert_eq(shuffled.size(), 5, "même taille");

    for value in original {
        assert_true(shuffled.contains(value), "chaque élément original est présent dans le mélange");
    }
}

func test_complexe() {
    let a = new Complexe(1, 2);
    let b = new Complexe(3, -4);

    let sum = a.add(b);
    assert_eq(sum.real, 4, "Complexe.add: partie réelle");
    assert_eq(sum.imaginaire, -2, "Complexe.add: partie imaginaire");

    let diff = a.subtract(b);
    assert_eq(diff.real, -2, "Complexe.subtract: partie réelle");
    assert_eq(diff.imaginaire, 6, "Complexe.subtract: partie imaginaire");

    let product = a.multiply(b);
    assert_eq(product.real, 11, "Complexe.multiply: partie réelle (1*3 - 2*-4)");
    assert_eq(product.imaginaire, 2, "Complexe.multiply: partie imaginaire (1*-4 + 2*3)");

    let conj = a.conjugate();
    assert_eq(conj.imaginaire, -2, "conjugate() inverse le signe de la partie imaginaire");

    assert_true(close(a.magnitude(), sqrt(5)), "magnitude == hypot(real, imaginaire)");

    assert_eq(a.to_string(), "1+2i", "to_string, partie imaginaire positive");
    assert_eq(b.to_string(), "3-4i", "to_string, partie imaginaire négative");
}

run_tests([
    ["constants", test_constants],
    ["reexported natives (accès qualifié)", test_reexported_natives_qualified_access],
    ["angles", test_angles],
    ["hyperbolic", test_hyperbolic],
    ["roots and distance", test_roots_and_distance],
    ["rounding and sign", test_rounding_and_sign],
    ["clamp", test_clamp],
    ["interpolation", test_interpolation],
    ["number theory", test_number_theory],
    ["factorial and fibonacci", test_factorial_and_fibonacci],
    ["factorial rejects negative", test_factorial_rejects_negative],
    ["combinatorics", test_combinatorics],
    ["aggregates", test_aggregates],
    ["median of empty array throws", test_median_empty_throws],
    ["random within bounds", test_random_within_bounds],
    ["shuffle is a permutation and pure", test_shuffle_is_a_permutation_and_does_not_mutate],
    ["Complexe", test_complexe],
]);
