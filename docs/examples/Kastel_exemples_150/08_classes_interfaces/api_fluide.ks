// ====================================================================
// Kastel — Interface fluide
// Notions : méthodes qui retournent self pour chaîner les appels
// Résultat attendu :
//   Total : 60
// ====================================================================

class Panier {
    private let total: int = 0;

    func ajouter(prix: int) -> Panier {
        self.total = self.total + prix;
        return self;
    }

    func remise(pourcent: int) -> Panier {
        self.total = self.total - idiv(self.total * pourcent, 100);
        return self;
    }

    func total_final() -> int {
        return self.total;
    }
}

let panier = new Panier();
println("Total : {}", panier.ajouter(50).ajouter(30).remise(25).total_final());
