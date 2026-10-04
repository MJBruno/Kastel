// ====================================================================
// Kastel — Valeur par défaut et fusion
// Notions : get_or, update
// Résultat attendu :
//   0
//   12
//   5
// ====================================================================

let stock = {"pommes": 10};

println(stock.get_or("poires", 0));   // 0 : la clé n'existe pas

stock.update({"poires": 5, "pommes": 12});

println(stock["pommes"]);
println(stock["poires"]);
