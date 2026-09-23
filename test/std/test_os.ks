// test/std/test_os.ks
//
// exit() n'est volontairement pas testé ici : l'appeler terminerait le
// processus de test lui-même.

import std.os;
from std.testing import assert_true, assert_eq, run_tests;

func test_name_and_arch_are_non_empty_strings() {
    assert_true(os.name().size() > 0, "os.name() renvoie une chaîne non vide");
    assert_true(os.arch().size() > 0, "os.arch() renvoie une chaîne non vide");
}

func test_cwd_is_a_non_empty_string() {
    assert_true(os.cwd().size() > 0, "os.cwd() renvoie une chaîne non vide");
}

func test_args_is_a_list() {
    assert_eq(type(os.args()), "list", "os.args() renvoie une liste");
    // L'exécutable lui-même occupe toujours la position 0 (comme sys.argv).
    assert_true(os.args().size() >= 1, "os.args() contient au moins l'exécutable");
}

func test_env_missing_variable_returns_none() {
    let value = os.env("KASTEL_STDLIB_TEST_UNDEFINED_VAR_XYZ_123");
    assert_eq(value, None, "une variable d'environnement absente renvoie None");
}

func test_clock_increases_over_time() {
    let before = os.clock();
    let i = 0;
    while i < 1000000 {
        i = i + 1;
    }
    let after = os.clock();

    assert_true(after >= before, "clock() ne recule jamais");
}

run_tests([
    ["name/arch are non-empty strings", test_name_and_arch_are_non_empty_strings],
    ["cwd is a non-empty string", test_cwd_is_a_non_empty_string],
    ["args is a list", test_args_is_a_list],
    ["env of a missing variable is None", test_env_missing_variable_returns_none],
    ["clock increases over time", test_clock_increases_over_time],
]);
