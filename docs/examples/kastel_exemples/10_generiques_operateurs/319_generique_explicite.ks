// ==================================================================
// Exemple 319 — Préciser le type à l'appel
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// f<type>(...) force le paramètre de type.
// ------------------------------------------------------------------
// Sortie attendue :
//   2.5
//   txt
// ==================================================================

func convertir<T>(x: T) -> T {
    return x;
}

println(convertir<float>(2.5));
println(convertir<str>("txt"));
