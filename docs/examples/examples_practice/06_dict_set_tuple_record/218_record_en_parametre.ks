// ==================================================================
// Exemple 218 — Record en paramètre
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// Le type d'un paramètre peut être un type de record.
// ------------------------------------------------------------------
// Sortie attendue :
//   25
// ==================================================================

type Point = { x: int, y: int };

func distance_carree(a: Point, b: Point) -> int {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    return dx * dx + dy * dy;
}

println(distance_carree({ x: 0, y: 0 }, { x: 3, y: 4 }));
