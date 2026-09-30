// ==================================================================
// Exemple 229 — Champs privés et encapsulation
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// private réserve l'accès à la classe elle-même ; on expose des méthodes.
// ------------------------------------------------------------------
// Sortie attendue :
//   100
// ==================================================================

class Compte {
    private let solde: int = 0;

    func deposer(montant: int) {
        if montant > 0 {
            self.solde += montant;
        }
    }

    func consulter() -> int {
        return self.solde;
    }
}

let c = new Compte();
c.deposer(100);
c.deposer(-5);        // ignoré : montant invalide
println(c.consulter());
// println(c.solde);  // refusé : solde est privé
