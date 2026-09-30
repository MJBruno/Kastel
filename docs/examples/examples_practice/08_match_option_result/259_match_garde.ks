// ==================================================================
// Exemple 259 — Garde avec if
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Un bras n'est pris que si sa condition est vraie.
// ------------------------------------------------------------------
// Sortie attendue :
//   négatif
//   zéro
//   pair positif
//   impair positif
// ==================================================================

func classer(n: int) -> str {
    match n {
        x if x < 0 => { return "négatif"; }
        0 => { return "zéro"; }
        x if x % 2 == 0 => { return "pair positif"; }
        _ => { return "impair positif"; }
    }
}

println(classer(-3));
println(classer(0));
println(classer(8));
println(classer(7));
