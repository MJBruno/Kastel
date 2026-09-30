// ==================================================================
// Exemple 451 — Banque avec Result
// Catégorie : Mini-projets
// ------------------------------------------------------------------
// Les opérations qui peuvent échouer renvoient un Result.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ok(100)
//   Ok(70)
//   Err(fonds insuffisants)
//   Err(montant invalide)
// ==================================================================

class Compte {
    private let solde: int = 0;

    func deposer(m: int) -> Result<int, str> {
        if m <= 0 {
            return Err("montant invalide");
        }
        self.solde += m;
        return Ok(self.solde);
    }

    func retirer(m: int) -> Result<int, str> {
        if m > self.solde {
            return Err("fonds insuffisants");
        }
        self.solde -= m;
        return Ok(self.solde);
    }
}

let c = new Compte();
println(c.deposer(100));
println(c.retirer(30));
println(c.retirer(500));
println(c.deposer(-5));
