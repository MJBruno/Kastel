// test/std/test_strings.ks

import std.strings;
from std.testing import assert_eq, assert_true, assert_false, run_tests;

func test_capitalize() {
    assert_eq(strings.capitalize("hello"), "Hello", "capitalize: cas simple");
    assert_eq(strings.capitalize(""), "", "capitalize: chaîne vide");
    assert_eq(strings.capitalize("a"), "A", "capitalize: un seul caractère");
    assert_eq(strings.capitalize("Already"), "Already", "capitalize: déjà capitalisé");
}

func test_pad_left() {
    assert_eq(strings.pad_left("7", 3, "0"), "007", "pad_left: complète à gauche");
    assert_eq(strings.pad_left("777", 3, "0"), "777", "pad_left: déjà à la largeur voulue");
    assert_eq(strings.pad_left("7777", 3, "0"), "7777", "pad_left: déjà plus large, inchangé");
}

func test_pad_right() {
    assert_eq(strings.pad_right("ab", 5, "-"), "ab---", "pad_right: complète à droite");
    assert_eq(strings.pad_right("abcde", 5, "-"), "abcde", "pad_right: déjà à la largeur voulue");
}

func test_is_palindrome() {
    assert_true(strings.is_palindrome("radar"), "radar est un palindrome");
    assert_true(strings.is_palindrome("Kayak"), "Kayak est un palindrome (insensible à la casse)");
    assert_false(strings.is_palindrome("hello"), "hello n'est pas un palindrome");
    assert_true(strings.is_palindrome(""), "la chaîne vide est un palindrome");
    assert_true(strings.is_palindrome("a"), "un seul caractère est un palindrome");
}

func test_count_occurrences() {
    assert_eq(strings.count_occurrences("banana", "an"), 2, "occurrences non chevauchantes");
    assert_eq(strings.count_occurrences("aaaa", "aa"), 2, "occurrences non chevauchantes (aaaa/aa)");
    assert_eq(strings.count_occurrences("hello", "z"), 0, "aucune occurrence");
    assert_eq(strings.count_occurrences("hello", ""), 0, "needle vide -> 0, pas de boucle infinie");
}

run_tests([
    ["capitalize", test_capitalize],
    ["pad_left", test_pad_left],
    ["pad_right", test_pad_right],
    ["is_palindrome", test_is_palindrome],
    ["count_occurrences", test_count_occurrences],
]);
