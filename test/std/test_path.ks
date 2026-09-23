// test/std/test_path.ks

import std.path;
import std.os;
from std.testing import assert_eq, assert_true, assert_false, run_tests;

func test_join() {
    let full = path.join(["dossier", "sous-dossier", "fichier.txt"]);

    assert_true(full.contains("dossier"), "join: contient le premier segment");
    assert_true(full.contains("fichier.txt"), "join: contient le dernier segment");
}

func test_join2() {
    assert_eq(path.join2("a", "b"), path.join(["a", "b"]), "join2 équivaut à join([a, b])");
}

func test_basename_dirname() {
    let full = path.join(["dossier", "sous-dossier", "fichier.txt"]);

    assert_eq(path.basename(full), "fichier.txt", "basename");
    assert_true(path.dirname(full).contains("sous-dossier"), "dirname contient le dossier parent");
}

func test_extension_and_stem() {
    let full = path.join(["dossier", "fichier.txt"]);

    assert_eq(path.extension(full), "txt", "extension sans le point");
    assert_eq(path.stem(full), "fichier", "stem sans l'extension");
}

func test_has_extension() {
    let full = path.join(["dossier", "fichier.txt"]);

    assert_true(path.has_extension(full, "txt"), "has_extension: correspond");
    assert_false(path.has_extension(full, "csv"), "has_extension: ne correspond pas");
}

func test_exists_and_is_dir_for_current_directory() {
    let cwd = os.cwd();

    assert_true(path.exists(cwd), "le répertoire courant existe");
    assert_true(path.is_dir(cwd), "le répertoire courant est un dossier");
    assert_false(path.is_file(cwd), "le répertoire courant n'est pas un fichier");
}

func test_exists_false_for_missing_path() {
    let missing = path.join([os.cwd(), "ce_chemin_ne_devrait_pas_exister_xyz_123"]);
    assert_false(path.exists(missing), "un chemin inexistant renvoie false");
}

run_tests([
    ["join", test_join],
    ["join2", test_join2],
    ["basename/dirname", test_basename_dirname],
    ["extension/stem", test_extension_and_stem],
    ["has_extension", test_has_extension],
    ["exists/is_dir for current directory", test_exists_and_is_dir_for_current_directory],
    ["exists is false for a missing path", test_exists_false_for_missing_path],
]);
