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

export func assert_true(condition: bool, message: str) -> None {
    if !condition {
        throw message;
    }
}

export func assert_false(condition: bool, message: str) -> None {
    if condition {
        throw message;
    }
}

export func assert_eq(actual: dynamic, expected: dynamic, message: str) -> None {
    if actual != expected {
        throw format("{} (attendu {}, obtenu {})", message, expected, actual);
    }
}

export func assert_not_eq(actual: dynamic, expected: dynamic, message: str) -> None {
    if actual == expected {
        throw format("{} (ne devait pas valoir {})", message, expected);
    }
}

// Compare deux floats a une tolerance pres (l'egalite exacte de floats
// est presque toujours le mauvais test : 0.1 + 0.2 != 0.3).
export func assert_almost_eq(actual: float, expected: float, tolerance: float, message: str) -> None {
    let diff = actual - expected;
    if diff < 0.0 {
        diff = -diff;
    }
    if diff > tolerance {
        throw format(
            "{} (attendu {} +/- {}, obtenu {})",
            message, expected, tolerance, actual
        );
    }
}

// Verifie que `action` (fonction sans argument) leve bien une
// exception. Echoue si `action` se termine normalement.
export func assert_throws(action: dynamic, message: str) -> None {
    try {
        action();
    } catch (_) {
        return;
    }
    throw format("{} (aucune exception levee)", message);
}

// Comme assert_eq, mais pour un Result<T, E> : echoue si c'est Err(...).
export func assert_ok(result: dynamic, message: str) -> None {
    match result {
        Ok(_) => {
            return;
        }
        Err(error) => {
            throw format("{} (Err inattendu: {})", message, error);
        }
    }
}

// Comme assert_ok, mais attend Err(...) : echoue si c'est Ok(...).
export func assert_err(result: dynamic, message: str) -> None {
    match result {
        Ok(value) => {
            throw format("{} (Ok inattendu: {})", message, value);
        }
        Err(_) => {
            return;
        }
    }
}

// `cases` est un tableau de paires [nom, fonction_sans_argument].
// Chaque fonction signale un échec via un `throw` (voir les
// assert_* ci-dessus). Renvoie `true` si tout est passé.
export func run_tests(cases: List<Tuple<str, dynamic>>) -> bool {
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
