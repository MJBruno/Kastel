import std.testing;

func test_addition() -> None {
    testing.assert_eq(2 + 3, 5, "2 + 3 doit donner 5");
}

func test_truth() -> None {
    testing.assert_true(true, "true attendu");
    testing.assert_false(false, "false attendu");
}

func test_float() -> None {
    testing.assert_almost_eq(0.3, 0.1 + 0.2, 0.000001, "precision");
}

let cases = [
    ("addition", test_addition),
    ("truth", test_truth),
    ("float", test_float)
];

let ok = testing.run_tests(cases);
println(ok);
println("std.testing: OK");
