// ==================================================================
// Exemple 321 — Borne Mul
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// Multiplier deux valeurs d'un même type numérique.
// ------------------------------------------------------------------
// Sortie attendue :
//   81
//   2.25
// ==================================================================

func carre<T: Mul>(x: T) -> T {
    return x * x;
}

println(carre(9));
println(carre(1.5));
