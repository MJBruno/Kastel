// ==================================================================
// Exemple 323 — Plusieurs bornes avec +
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// T: Add + Eq : additionner et comparer.
// ------------------------------------------------------------------
// Sortie attendue :
//   8
//   4
// ==================================================================

func double_si_egal<T: Add + Eq>(a: T, b: T) -> T {
    if a == b {
        return a + b;
    }
    return a;
}

println(double_si_egal(4, 4));
println(double_si_egal(4, 5));
