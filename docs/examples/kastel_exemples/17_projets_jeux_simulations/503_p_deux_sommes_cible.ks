// ==================================================================
// Exemple 503 — Compte est bon simplifié
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : trouver deux nombres dont la somme vaut la cible.
// ------------------------------------------------------------------
// Sortie attendue :
//   Some((9, 5))
//   None
// ==================================================================

func deux_sommes(v: List<int>, cible: int) -> Option<Tuple<int, int>> {
    let vus = Set();
    for x in v {
        if vus.contains(cible - x) {
            return Some((cible - x, x));
        }
        vus.add(x);
    }
    return None;
}

println(deux_sommes([3, 8, 1, 9, 5], 14));
println(deux_sommes([3, 8, 1], 100));
