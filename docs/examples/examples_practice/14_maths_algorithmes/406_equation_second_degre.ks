// ==================================================================
// Exemple 406 — Équation du second degré
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Discriminant puis racines, avec Result pour le cas sans solution.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ok((2.0, 1.0))
//   Err(pas de racine réelle)
// ==================================================================

func resoudre(a: float, b: float, c: float) -> Result<Tuple<float, float>, str> {
    let d = b * b - 4.0 * a * c;
    if d < 0.0 {
        return Err("pas de racine réelle");
    }
    let r = sqrt(d);
    return Ok(((-b + r) / (2.0 * a), (-b - r) / (2.0 * a)));
}

println(resoudre(1.0, -3.0, 2.0));
println(resoudre(1.0, 0.0, 1.0));
