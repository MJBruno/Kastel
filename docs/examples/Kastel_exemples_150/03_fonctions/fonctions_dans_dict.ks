// ====================================================================
// Kastel — Fonctions dans un dict
// Notions : dispatch par nom d'opération
// Résultat attendu :
//   7
//   12
// ====================================================================

let operations = {
    "add": (a, b) => a + b,
    "mul": (a, b) => a * b
};

println(operations["add"](3, 4));
println(operations["mul"](3, 4));
