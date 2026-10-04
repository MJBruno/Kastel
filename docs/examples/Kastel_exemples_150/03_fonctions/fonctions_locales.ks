// ====================================================================
// Kastel — Fonctions locales
// Notions : func déclarée dans une autre fonction
// Résultat attendu :
//   11
// ====================================================================

func calculer(x) {
    func doubler(n) {
        return n * 2;
    }

    func incrementer(n) {
        return n + 1;
    }

    return incrementer(doubler(x));
}

println(calculer(5));
