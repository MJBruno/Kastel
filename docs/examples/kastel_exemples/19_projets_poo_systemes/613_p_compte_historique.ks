// ==================================================================
// Exemple 613 — Compte avec historique d'opérations
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : le solde se recalcule en rejouant toutes les opérations.
// ------------------------------------------------------------------
// Sortie attendue :
//   solde : 120
//   opérations : 3
// ==================================================================

class Compte {
    private let operations: List<int> = [];

    func deposer(m: int) { self.operations.add(m); }
    func retirer(m: int) { self.operations.add(-m); }

    func solde() -> int {
        let s = 0;
        for o in self.operations { s += o; }
        return s;
    }

    func nombre() -> int { return self.operations.size(); }
}

let c = new Compte();
c.deposer(100);
c.retirer(30);
c.deposer(50);
println("solde : {}", c.solde());
println("opérations : {}", c.nombre());
