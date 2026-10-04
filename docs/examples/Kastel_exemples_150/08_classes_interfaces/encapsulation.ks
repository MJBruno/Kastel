// ====================================================================
// Kastel — Champs privés
// Notions : private let, getter, validation par exception
// Résultat attendu :
//   Solde : 150
//   Refusé : montant invalide
//   Refusé : fonds insuffisants
//   Solde final : 100
// ====================================================================

class Compte {
    private let solde: int = 0;

    func deposer(montant: int) {
        if montant <= 0 {
            throw "montant invalide";
        }
        self.solde = self.solde + montant;
    }

    func retirer(montant: int) {
        if montant > self.solde {
            throw "fonds insuffisants";
        }
        self.solde = self.solde - montant;
    }

    func solde_actuel() -> int {
        return self.solde;
    }
}

let compte = new Compte();
compte.deposer(150);
println("Solde : {}", compte.solde_actuel());

try {
    compte.deposer(-5);
} catch (e) {
    println("Refusé : {}", e);
}

try {
    compte.retirer(1000);
} catch (e) {
    println("Refusé : {}", e);
}

compte.retirer(50);
println("Solde final : {}", compte.solde_actuel());

// compte.solde  -> erreur de compilation : champ privé.
