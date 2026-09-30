// ==================================================================
// Exemple 126 — Générique sur une liste
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// List<T> relie le type des éléments au type du résultat.
// ------------------------------------------------------------------
// Sortie attendue :
//   10
//   a
// ==================================================================

func premier<T>(items: List<T>) -> T {
    return items[0];
}

let n: int = premier([10, 20, 30]);
let s: str = premier(["a", "b"]);
println(n);
println(s);
