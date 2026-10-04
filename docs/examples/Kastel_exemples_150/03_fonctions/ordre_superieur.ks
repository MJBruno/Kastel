// ====================================================================
// Kastel — Fonctions d'ordre supérieur
// Notions : passer et retourner des fonctions
// Résultat attendu :
//   16
//   21
// ====================================================================

func appliquer_deux_fois(f, x) {
    return f(f(x));
}

func composer(f, g) {
    return x => f(g(x));
}

println(appliquer_deux_fois(x => x * 2, 4));

let plus_un_puis_triple = composer(x => x * 3, x => x + 1);
println(plus_un_puis_triple(6));
