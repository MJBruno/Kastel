// ==================================================================
// Exemple 486 — Cavalier d'échecs : coups possibles
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : lister les 8 sauts en L qui restent sur l'échiquier.
// ------------------------------------------------------------------
// Sortie attendue :
//   2
//   8
//   [(1, 2), (2, 1)]
// ==================================================================

func coups(x: int, y: int) -> List<Tuple<int, int>> {
    let sauts = [(1,2),(2,1),(-1,2),(-2,1),(1,-2),(2,-1),(-1,-2),(-2,-1)];
    let res = [];
    for s in sauts {
        let nx = x + s[0];
        let ny = y + s[1];
        if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
            res.add((nx, ny));
        }
    }
    return res;
}

println(coups(0, 0).size());
println(coups(3, 3).size());
println(coups(0, 0));
