// ==================================================================
// Exemple 322 — Borne Ord : comparer
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// T: Ord autorise <, >, <=, >=.
// ------------------------------------------------------------------
// Sortie attendue :
//   8
//   2.5
// ==================================================================

func plus_grand<T: Ord>(a: T, b: T) -> T {
    if a > b {
        return a;
    }
    return b;
}

println(plus_grand(3, 8));
println(plus_grand(2.5, 1.5));
