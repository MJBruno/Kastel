// ====================================================================
// Kastel — Retours anticipés
// Notions : return tôt pour éviter l'imbrication
// Résultat attendu :
//   -4 : negatif
//   0 : zero
//   9 : positif
// ====================================================================

func classer(n) {
    if n < 0 {
        return "negatif";
    }
    if n == 0 {
        return "zero";
    }
    return "positif";
}

for n in [-4, 0, 9] {
    println("{} : {}", n, classer(n));
}
