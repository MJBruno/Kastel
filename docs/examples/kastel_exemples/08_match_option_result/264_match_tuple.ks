// ==================================================================
// Exemple 264 — Décomposer un tuple
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Un motif (x, y) lie les deux composantes.
// ------------------------------------------------------------------
// Sortie attendue :
//   origine
//   sur l'axe x en 5
//   sur l'axe y en 7
//   point 2,3
// ==================================================================

func decrire(p: Tuple<int, int>) -> str {
    match p {
        (0, 0) => { return "origine"; }
        (x, 0) => { return "sur l'axe x en " + str(x); }
        (0, y) => { return "sur l'axe y en " + str(y); }
        (x, y) => { return "point " + str(x) + "," + str(y); }
    }
}

println(decrire((0, 0)));
println(decrire((5, 0)));
println(decrire((0, 7)));
println(decrire((2, 3)));
