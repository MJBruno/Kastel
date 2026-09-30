// ==================================================================
// Exemple 320 — Borne Add : accepter l'addition
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// T: Add exige que le type sache s'additionner.
// ------------------------------------------------------------------
// Sortie attendue :
//   5
//   4.0
// ==================================================================

func additionner<T: Add>(a: T, b: T) -> T {
    return a + b;
}

println(additionner(2, 3));
println(additionner(1.5, 2.5));
