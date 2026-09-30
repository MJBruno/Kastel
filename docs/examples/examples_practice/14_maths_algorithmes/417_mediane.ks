// ==================================================================
// Exemple 417 — Médiane
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Trier puis prendre l'élément du milieu.
// ------------------------------------------------------------------
// Sortie attendue :
//   5.0
//   2.5
// ==================================================================

func mediane(v: List<int>) -> float {
    let c = v.copy();
    c.sort();
    let n = c.size();
    if n % 2 == 1 {
        return float(c[idiv(n, 2)]);
    }
    return (c[idiv(n, 2) - 1] + c[idiv(n, 2)]) / 2;
}

println(mediane([7, 1, 3, 5, 9]));
println(mediane([4, 1, 3, 2]));
