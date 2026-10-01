// ==================================================================
// Exemple 579 — Distributeur de billets
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : solde, plafond journalier et opérations qui renvoient un Result.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ok(900)
//   Ok(750)
//   Err(plafond journalier atteint)
// ==================================================================

class Guichet {
    private let solde: int = 1000;
    private let plafond: int = 300;
    private let retire_jour: int = 0;

    func retirer(m: int) -> Result<int, str> {
        if m > self.solde {
            return Err("solde insuffisant");
        }
        if self.retire_jour + m > self.plafond {
            return Err("plafond journalier atteint");
        }
        self.solde = self.solde - m;
        self.retire_jour = self.retire_jour + m;
        return Ok(self.solde);
    }
}

let g = new Guichet();
println(g.retirer(100));
println(g.retirer(150));
println(g.retirer(100));
