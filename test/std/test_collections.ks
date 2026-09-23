// test/std/test_collections.ks

import std.collections;
from std.testing import assert_eq, assert_true, assert_false, run_tests;

func test_enumerate() {
    let result = collections.enumerate(["a", "b", "c"]);

    assert_eq(result.size(), 3, "enumerate: taille");
    assert_eq(result.get(0).get(0), 0, "enumerate: index 0");
    assert_eq(result.get(0).get(1), "a", "enumerate: valeur 0");
    assert_eq(result.get(2).get(0), 2, "enumerate: index 2");
    assert_eq(result.get(2).get(1), "c", "enumerate: valeur 2");
}

func test_enumerate_empty() {
    assert_eq(collections.enumerate([]).size(), 0, "enumerate([]) est vide");
}

func test_zip() {
    let result = collections.zip([1, 2, 3], ["a", "b", "c"]);

    assert_eq(result.size(), 3, "zip: taille = min des deux");
    assert_eq(result.get(1).get(0), 2, "zip: paire[1][0]");
    assert_eq(result.get(1).get(1), "b", "zip: paire[1][1]");
}

func test_zip_uneven_lengths() {
    let result = collections.zip([1, 2, 3, 4, 5], ["a", "b"]);
    assert_eq(result.size(), 2, "zip: s'arrête à la longueur la plus courte");
}

// Reproduction du bug corrigé : flatten() comparait type(item) à "array",
// alors que type() renvoie "list" depuis le renommage Array -> List.
// Sans le correctif, ce test échoue (flatten ne fait rien).
func test_flatten() {
    let nested = [1, [2, 3], [4, [5, 6]], 7];
    let flat = collections.flatten(nested);

    assert_eq(flat.size(), 7, "flatten: aplatit tous les niveaux");
    assert_eq(flat.get(0), 1, "flatten[0]");
    assert_eq(flat.get(1), 2, "flatten[1]");
    assert_eq(flat.get(4), 5, "flatten[4]");
    assert_eq(flat.get(6), 7, "flatten[6]");
}

func test_flatten_already_flat() {
    let flat = collections.flatten([1, 2, 3]);
    assert_eq(flat.size(), 3, "flatten d'un tableau déjà plat ne change rien");
}

func test_flatten_empty_sublists() {
    let flat = collections.flatten([[], [1], [], [2, 3], []]);
    assert_eq(flat.size(), 3, "flatten ignore les sous-tableaux vides");
}

func test_unique() {
    let result = collections.unique([1, 2, 2, 3, 1, 4, 3]);

    assert_eq(result.size(), 4, "unique: dédoublonne");
    assert_eq(result.get(0), 1, "unique: garde l'ordre de première apparition");
    assert_eq(result.get(1), 2, "unique[1]");
    assert_eq(result.get(2), 3, "unique[2]");
    assert_eq(result.get(3), 4, "unique[3]");
}

func test_chunk() {
    let result = collections.chunk([1, 2, 3, 4, 5], 2);

    assert_eq(result.size(), 3, "chunk: nombre de groupes");
    assert_eq(result.get(0).size(), 2, "chunk: taille du premier groupe");
    assert_eq(result.get(2).size(), 1, "chunk: dernier groupe partiel");
    assert_eq(result.get(2).get(0), 5, "chunk: contenu du dernier groupe");
}

func test_chunk_exact_multiple() {
    let result = collections.chunk([1, 2, 3, 4], 2);
    assert_eq(result.size(), 2, "chunk: pas de groupe partiel vide quand ça tombe juste");
}

func test_range_array() {
    let result = collections.range_array(5);

    assert_eq(result.size(), 5, "range_array: taille");
    assert_eq(result.get(0), 0, "range_array[0]");
    assert_eq(result.get(4), 4, "range_array[4]");
}

func test_range_array_zero() {
    assert_eq(collections.range_array(0).size(), 0, "range_array(0) est vide");
}

run_tests([
    ["enumerate", test_enumerate],
    ["enumerate([]) is empty", test_enumerate_empty],
    ["zip", test_zip],
    ["zip with uneven lengths", test_zip_uneven_lengths],
    ["flatten (regression: type() renvoie 'list')", test_flatten],
    ["flatten already-flat array", test_flatten_already_flat],
    ["flatten ignores empty sublists", test_flatten_empty_sublists],
    ["unique", test_unique],
    ["chunk", test_chunk],
    ["chunk exact multiple", test_chunk_exact_multiple],
    ["range_array", test_range_array],
    ["range_array(0)", test_range_array_zero],
]);
