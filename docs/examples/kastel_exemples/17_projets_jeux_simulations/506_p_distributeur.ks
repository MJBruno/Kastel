// ==================================================================
// Exemple 506 — Distributeur de boissons
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : un distributeur avec crédit, stock et rendu de monnaie.
// ------------------------------------------------------------------
// Sortie attendue :
//   Err(crédit insuffisant)
//   Err(épuisé)
//   Ok(30)
// ==================================================================

class Distributeur {
    private let stock = {"cafe": 2, "the": 0};
    private let prix = {"cafe": 120, "the": 100};
    private let credit: int = 0;

    func inserer(c: int) { self.credit = self.credit + c; }

    func choisir(produit: str) -> Result<int, str> {
        if self.stock[produit] == 0 { return Err("épuisé"); }
        if self.credit < self.prix[produit] { return Err("crédit insuffisant"); }
        let monnaie = self.credit - self.prix[produit];
        self.stock[produit] = self.stock[produit] - 1;
        self.credit = 0;
        return Ok(monnaie);
    }
}

let d = new Distributeur();
println(d.choisir("cafe"));
d.inserer(100);
println(d.choisir("the"));
d.inserer(50);
println(d.choisir("cafe"));
