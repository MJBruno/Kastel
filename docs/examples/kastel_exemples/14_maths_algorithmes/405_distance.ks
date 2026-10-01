// ==================================================================
// Exemple 405 — Distance entre deux points
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Le théorème de Pythagore.
// ------------------------------------------------------------------
// Sortie attendue :
//   5.0
// ==================================================================

func distance(x1: float, y1: float, x2: float, y2: float) -> float {
    let dx = x2 - x1;
    let dy = y2 - y1;
    return sqrt(dx * dx + dy * dy);
}

println(distance(0.0, 0.0, 3.0, 4.0));
