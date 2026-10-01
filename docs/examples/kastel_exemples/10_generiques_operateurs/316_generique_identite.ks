// ==================================================================
// Exemple 316 — Fonction générique
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// <T> : le même code pour tous les types, avec le type conservé.
// ------------------------------------------------------------------
// Sortie attendue :
//   7
//   ok
// ==================================================================

func identite<T>(x: T) -> T {
    return x;
}

let a: int = identite(7);
let b: str = identite("ok");
println(a);
println(b);
