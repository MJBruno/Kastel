// ====================================================================
// Kastel — Dictionnaires : les bases
// Notions : littéral à clés chaînes, index, contains, set, get, remove
// Résultat attendu :
//   10
//   3
//   true
//   2
//   7
// ====================================================================

let stock = {"pommes": 10, "poires": 4};

println(stock["pommes"]);

stock["cerises"] = 25;
println(stock.size());
println(stock.contains("poires"));   // contains teste une CLÉ

stock.remove("poires");
println(stock.size());

stock.set("pommes", 7);
println(stock.get("pommes"));
