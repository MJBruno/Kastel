// test/std/test_json.ks

import std.json;
import std.path;
import std.os;
from std.testing import assert_eq, assert_true, assert_false, run_tests;

func test_encode_decode_record_roundtrip() {
    let original = { name: "Ada", age: 36 };
    let encoded = json.encode(original);
    let decoded = json.decode(encoded);

    assert_eq(decoded.get("name"), "Ada", "roundtrip: champ name");
    assert_eq(decoded.get("age"), 36, "roundtrip: champ age");
}

func test_decode_literal_string() {
    let data = json.decode("{\"x\": 1, \"y\": [1, 2, 3]}");

    assert_eq(data.get("x"), 1, "decode: champ scalaire");
    assert_eq(data.get("y").size(), 3, "decode: tableau imbriqué");
    assert_eq(data.get("y").get(2), 3, "decode: élément du tableau imbriqué");
}

func test_encode_array() {
    let encoded = json.encode([1, 2, 3]);
    let decoded = json.decode(encoded);

    assert_eq(decoded.size(), 3, "encode/decode d'un tableau top-level");
    assert_eq(decoded.get(1), 2, "élément du tableau");
}

func test_read_write_file_roundtrip() {
    let temp = path.join([os.cwd(), "kastel_stdlib_test_json_tmp.json"]);

    json.write_file(temp, { greeting: "hello", count: 3 });
    let loaded = json.read_file(temp);

    assert_eq(loaded.get("greeting"), "hello", "read_file après write_file: champ string");
    assert_eq(loaded.get("count"), 3, "read_file après write_file: champ entier");

    file_delete(temp);
    assert_false(path.exists(temp), "le fichier temporaire est bien supprimé après le test");
}

run_tests([
    ["encode/decode record roundtrip", test_encode_decode_record_roundtrip],
    ["decode literal string", test_decode_literal_string],
    ["encode array", test_encode_array],
    ["read_file/write_file roundtrip", test_read_write_file_roundtrip],
]);
