// examples/function_overloads_demo.ks
//
// Surcharge de FONCTIONS GLOBALES par nombre de paramètres (arité).
// Comme pour les méthodes et les constructeurs : même nom + même arité =
// erreur ; le choix se fait à l'appel.
// Lancer avec : kastel examples/function_overloads_demo.ks

func area(side: int) -> int {
    return side * side;
}

func area(width: int, height: int) -> int {
    return width * height;
}

func format_price(amount) {
    return str(amount) + " EUR";
}

func format_price(amount, currency) {
    return str(amount) + " " + currency;
}

println(area(3));                       // 9
println(area(2, 5));                    // 10

println(format_price(12));              // 12 EUR
println(format_price(12, "USD"));       // 12 USD

// Un ensemble de surcharges se passe comme une valeur : l'arité des
// arguments choisit la surcharge.
let f = area;

println(f(4));                          // 16
println(f(4, 6));                       // 24

// Refusés (à décommenter pour voir l'erreur) :
//   area(1, 2, 3);                     // aucune surcharge à 3 paramètres
//   func area(x: int) -> int { ... }   // 'area' à 1 paramètre déjà déclarée
