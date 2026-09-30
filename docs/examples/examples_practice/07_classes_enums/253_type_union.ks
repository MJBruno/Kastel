// ==================================================================
// Exemple 253 — Type union
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// int | float accepte l'un ou l'autre.
// ------------------------------------------------------------------
// Sortie attendue :
//   6
//   5.0
// ==================================================================

type Nombre = int | float;

func doubler(n: Nombre) -> Nombre {
    return n * 2;
}

println(doubler(3));
println(doubler(2.5));
