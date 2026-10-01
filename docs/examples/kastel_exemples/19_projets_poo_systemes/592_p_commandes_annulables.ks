// ==================================================================
// Exemple 592 — Patron Commande
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : des actions enregistrées qu'on peut défaire.
// ------------------------------------------------------------------
// Sortie attendue :
//   8
//   5
//   0
// ==================================================================

class Calculateur {
    let total: int = 0;
    private let historique: List<int> = [];

    func ajouter(n: int) {
        self.total = self.total + n;
        self.historique.add(n);
    }

    func annuler() {
        if !self.historique.is_empty() {
            self.total = self.total - self.historique.pop();
        }
    }
}

let c = new Calculateur();
c.ajouter(5);
c.ajouter(3);
println(c.total);
c.annuler();
println(c.total);
c.annuler();
println(c.total);
