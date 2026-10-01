// ==================================================================
// Exemple 263 — Liste de taille exacte
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// [a, b] ne correspond qu'aux listes de deux éléments.
// ------------------------------------------------------------------
// Sortie attendue :
//   vide
//   un : 4
//   deux : 9
//   beaucoup
// ==================================================================

func paire(v: List<int>) -> str {
    match v {
        [] => { return "vide"; }
        [a] => { return "un : " + str(a); }
        [a, b] => { return "deux : " + str(a + b); }
        _ => { return "beaucoup"; }
    }
}

println(paire([]));
println(paire([4]));
println(paire([4, 5]));
println(paire([1, 2, 3]));
