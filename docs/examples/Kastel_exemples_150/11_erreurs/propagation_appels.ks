// ====================================================================
// Kastel — Propagation à travers les appels
// Notions : une erreur remonte jusqu'au premier catch
// Résultat attendu :
//   attrapée en haut : valeur négative
// ====================================================================

func niveau3(n) {
    if n < 0 {
        throw "valeur négative";
    }
    return n;
}

func niveau2(n) { return niveau3(n) * 2; }
func niveau1(n) { return niveau2(n) + 1; }

try {
    println(niveau1(-4));
} catch (e) {
    println("attrapée en haut : {}", e);
}
