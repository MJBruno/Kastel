// ====================================================================
// Kastel — Itérateur personnalisé
// Notions : Iterator<T> avec next() et has_next(), puis map/filter/take/collect
// Résultat attendu :
//   [4, 6]
// ====================================================================

class Compteur: Iterator<int> {
    let courant: int;
    let limite: int;

    func initialize(limite: int) {
        self.courant = 0;
        self.limite = limite;
    }

    func next() -> int {
        let valeur = self.courant;
        self.courant = self.courant + 1;
        return valeur;
    }

    func has_next() -> bool {
        return self.courant < self.limite;
    }
}

let it: Iterator<int> = new Compteur(5);

let valeurs = it
    .map((v) => v * 2)
    .filter((v) => v >= 4)
    .take(2)
    .collect();

println(valeurs);
