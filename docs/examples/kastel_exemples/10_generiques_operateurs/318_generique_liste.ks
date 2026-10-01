// ==================================================================
// Exemple 318 — Générique sur List<T>
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// Le type des éléments se retrouve dans le type de retour.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
//   y
// ==================================================================

func dernier<T>(v: List<T>) -> T {
    return v[v.size() - 1];
}

let a: int = dernier([1, 2, 3]);
let b: str = dernier(["x", "y"]);
println(a);
println(b);
