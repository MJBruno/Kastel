// ==================================================================
// Exemple 217 — Nommer un type de record
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// type Nom = { ... } donne un nom réutilisable.
// ------------------------------------------------------------------
// Sortie attendue :
//   0
// ==================================================================

type Point = { x: int, y: int };

func origine() -> Point {
    return { x: 0, y: 0 };
}

let p: Point = origine();
println(p.x + p.y);
