// ==================================================================
// Exemple 317 — Paire générique
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// Deux paramètres de type indépendants.
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   y
// ==================================================================

func premier_de<A, B>(a: A, b: B) -> A {
    return a;
}

println(premier_de(1, "x"));
println(premier_de("y", 2));
