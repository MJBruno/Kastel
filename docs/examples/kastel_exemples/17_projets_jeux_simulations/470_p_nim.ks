// ==================================================================
// Exemple 470 — Jeu de Nim : stratégie gagnante
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : à Nim, l'état est gagnant si le XOR des tas n'est pas nul.
// ------------------------------------------------------------------
// Sortie attendue :
//   Some((0, 2))
//   None
// ==================================================================

func xor_tas(tas: List<int>) -> int {
    let x = 0;
    for t in tas {
        x = x ^ t;
    }
    return x;
}

func meilleur_coup(tas: List<int>) -> Option<Tuple<int, int>> {
    let x = xor_tas(tas);
    if x == 0 {
        return None;                 // position perdante
    }
    for i in range(tas.size()) {
        let cible = tas[i] ^ x;
        if cible < tas[i] {
            return Some((i, tas[i] - cible));   // (tas, nombre à retirer)
        }
    }
    return None;
}

println(meilleur_coup([3, 4, 5]));
println(meilleur_coup([1, 2, 3]));
