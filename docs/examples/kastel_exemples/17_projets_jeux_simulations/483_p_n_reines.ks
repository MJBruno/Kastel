// ==================================================================
// Exemple 483 — Problème des N reines
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : compter les placements de N reines sans prise (retour arrière).
// ------------------------------------------------------------------
// Sortie attendue :
//   2
//   4
// ==================================================================

func compter(n: int, ligne: int, colonnes: List<int>) -> int {
    if ligne == n {
        return 1;
    }
    let total = 0;
    for c in range(n) {
        let ok = true;
        for r in range(ligne) {
            let cr = colonnes[r];
            if cr == c || cr - c == r - ligne || cr - c == ligne - r {
                ok = false;
                break;
            }
        }
        if ok {
            colonnes.add(c);
            total += compter(n, ligne + 1, colonnes);
            colonnes.pop();
        }
    }
    return total;
}

println(compter(4, 0, []));
println(compter(6, 0, []));
