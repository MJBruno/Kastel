// ==================================================================
// Exemple 328 — Surcharger + avec Add
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// La classe déclare l'interface Add et fournit la méthode add.
// ------------------------------------------------------------------
// Sortie attendue :
//   150
// ==================================================================

class Argent : Add {
    let montant: int;

    func initialize(m: int) {
        self.montant = m;
    }

    func add(autre: Argent) -> Argent {
        return new Argent(self.montant + autre.montant);
    }
}

let total = new Argent(100) + new Argent(50);
println(total.montant);
