// test/std/run_all.ks
//
// Exécute toutes les suites de tests de la bibliothèque standard.
// Chaque test_*.ks appelle run_tests(...) au niveau module ; importer
// le fichier suffit donc à l'exécuter (effet de bord à l'import).
//
// Usage : kastel test/std/run_all.ks
//
// Remarque : chaque suite imprime son propre résumé
// ("N passed, M failed"). Ce fichier ne calcule pas de statut de sortie
// agrégé — si vous voulez un code de sortie non nul en cas d'échec (pour
// un hook CI), transformez chaque test_*.ks pour exposer
// `export func run() { ... return run_tests([...]); }` au lieu d'un
// appel au niveau module, puis sommez les résultats ici avec os.exit(1)
// si l'un d'eux est faux.

println("== std.math ==");
import "test_math.ks" as _test_math;

println("");
println("== std.collections ==");
import "test_collections.ks" as _test_collections;

println("");
println("== std.strings ==");
import "test_strings.ks" as _test_strings;

println("");
println("== std.datetime ==");
import "test_datetime.ks" as _test_datetime;

println("");
println("== std.json ==");
import "test_json.ks" as _test_json;

println("");
println("== std.path ==");
import "test_path.ks" as _test_path;

println("");
println("== std.os ==");
import "test_os.ks" as _test_os;

println("");
println("== std.file ==");
import "test_file.ks" as _test_file;
