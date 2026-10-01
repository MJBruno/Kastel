// ==================================================================
// Exemple 475 — Yams : calculer un score
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : somme d'une face, brelan, carré et yams sur 5 dés.
// ------------------------------------------------------------------
// Sortie attendue :
//   12
//   5
//   false
//   true
// ==================================================================

func occurrences(des: List<int>, face: int) -> int {
    let n = 0;
    for d in des { if d == face { n += 1; } }
    return n;
}

func score_face(des: List<int>, face: int) -> int {
    return occurrences(des, face) * face;
}

func yams(des: List<int>) -> bool {
    for d in des { if d != des[0] { return false; } }
    return true;
}

let des = [3, 3, 3, 5, 3];
println(score_face(des, 3));
println(score_face(des, 5));
println(yams(des));
println(yams([6, 6, 6, 6, 6]));
