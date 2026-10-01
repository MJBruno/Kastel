// ==================================================================
// Exemple 270 — and_then : enchaîner des Option
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// and_then attend une fonction qui renvoie elle-même un Option.
// ------------------------------------------------------------------
// Sortie attendue :
//   Some(4)
//   None
//   Some(2)
// ==================================================================

func moitie(n: int) -> Option<int> {
    if n % 2 == 0 {
        return Some(idiv(n, 2));
    }
    return None;
}

println(Some(8).and_then(moitie));
println(Some(7).and_then(moitie));
println(Some(8).and_then(moitie).and_then(moitie));
