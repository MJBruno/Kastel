// std/testing.ks
//
// Mini framework de test pour Kastel : assertions + petit runner.
//
// Usage typique :
//
//   from std.testing import assert_eq, run_tests;
//
//   func test_addition() {
//       assert_eq(2 + 2, 4, "2 + 2 doit valoir 4");
//   }
//
//   run_tests([
//       ["addition", test_addition],
//   ]);

export func assert_true(condition, message) {
    if !condition {
        throw message;
    }
}

export func assert_false(condition, message) {
    if condition {
        throw message;
    }
}

export func assert_eq(actual, expected, message) {
    if actual != expected {
        throw format("{} (attendu {}, obtenu {})", message, expected, actual);
    }
}

export func assert_not_eq(actual, expected, message) {
    if actual == expected {
        throw format("{} (ne devait pas valoir {})", message, expected);
    }
}

// `cases` est un tableau de paires [nom, fonction_sans_argument].
// Chaque fonction signale un échec via un `throw` (voir les
// assert_* ci-dessus). Renvoie `true` si tout est passé.
export func run_tests(cases) {
    let passed = 0;
    let failed = 0;

    for entry in cases {
        let name = entry.get(0);
        let test_func = entry.get(1);

        try {
            test_func();
            passed = passed + 1;
            println(format("  ok   {}", name));
        } catch (error) {
            failed = failed + 1;
            println(format("  FAIL {} - {}", name, error));
        }
    }

    println(format("{} passed, {} failed", passed, failed));

    return failed == 0;
}
