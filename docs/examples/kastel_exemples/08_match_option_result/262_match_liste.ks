// ==================================================================
// Exemple 262 — Décomposer une liste
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// [a, b, ..] nomme les premiers éléments et ignore le reste.
// ------------------------------------------------------------------
// Sortie attendue :
//   30
//   -1
// ==================================================================

func debut(v: List<int>) -> int {
    match v {
        [a, b, ..] => { return a + b; }
        _ => { return -1; }
    }
}

println(debut([10, 20, 30]));
println(debut([5]));
