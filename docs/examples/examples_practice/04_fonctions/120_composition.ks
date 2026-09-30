// ==================================================================
// Exemple 120 — Composer deux fonctions
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// compose(f, g)(x) = f(g(x)).
// ------------------------------------------------------------------
// Sortie attendue :
//   12
// ==================================================================

func compose(f, g) {
    return x => f(g(x));
}

let inc = x => x + 1;
let dbl = x => x * 2;
let inc_puis_dbl = compose(dbl, inc);   // dbl(inc(x))
println(inc_puis_dbl(5));
