// ====================================================================
// Kastel — Retourner plusieurs valeurs
// Notions : tuple en retour, get(i)
// Résultat attendu :
//   min = 1, max = 9
// ====================================================================

func min_max(valeurs) {
    let bas = valeurs[0];
    let haut = valeurs[0];

    for v in valeurs {
        if v < bas {
            bas = v;
        }
        if v > haut {
            haut = v;
        }
    }

    return (bas, haut);
}

let resultat = min_max([4, 9, 1, 7]);
println("min = {}, max = {}", resultat.get(0), resultat.get(1));
