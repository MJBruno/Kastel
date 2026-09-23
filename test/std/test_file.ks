// test/std/test_file.ks

import std.file;
import std.path;
import std.os;
from std.testing import assert_eq, assert_true, assert_false, run_tests;

func temp_path() {
    return path.join([os.cwd(), "kastel_stdlib_test_file_tmp.txt"]);
}

func test_write_read_roundtrip() {
    let temp = temp_path();

    file.write(temp, "hello");
    assert_eq(file.read(temp), "hello", "read renvoie exactement ce qui a été écrit");

    file.delete(temp);
}

func test_write_overwrites_existing_content() {
    let temp = temp_path();

    file.write(temp, "first");
    file.write(temp, "second");
    assert_eq(file.read(temp), "second", "write remplace le contenu précédent, ne l'ajoute pas");

    file.delete(temp);
}

func test_append_adds_to_existing_content() {
    let temp = temp_path();

    file.write(temp, "hello");
    file.append(temp, " world");
    assert_eq(file.read(temp), "hello world", "append ajoute à la suite du contenu existant");

    file.delete(temp);
}

func test_exists_and_delete() {
    let temp = temp_path();

    assert_false(file.exists(temp), "le fichier n'existe pas encore");

    file.write(temp, "x");
    assert_true(file.exists(temp), "le fichier existe après write");

    file.delete(temp);
    assert_false(file.exists(temp), "le fichier n'existe plus après delete");
}

func test_size() {
    let temp = temp_path();

    file.write(temp, "12345");
    assert_eq(file.size(temp), 5, "size renvoie le nombre d'octets écrits");

    file.delete(temp);
}

func test_read_lines() {
    let temp = temp_path();

    file.write(temp, "a\nb\nc");
    let lines = file.read_lines(temp);

    assert_eq(lines.size(), 3, "read_lines découpe sur les retours à la ligne");
    assert_eq(lines.get(0), "a", "read_lines[0]");
    assert_eq(lines.get(2), "c", "read_lines[2]");

    file.delete(temp);
}

run_tests([
    ["write/read roundtrip", test_write_read_roundtrip],
    ["write overwrites existing content", test_write_overwrites_existing_content],
    ["append adds to existing content", test_append_adds_to_existing_content],
    ["exists and delete", test_exists_and_delete],
    ["size", test_size],
    ["read_lines", test_read_lines],
]);
