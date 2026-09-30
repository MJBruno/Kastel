// ==================================================================
// Exemple 241 — Une pile (classe)
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Encapsuler une liste derrière une interface simple.
// ------------------------------------------------------------------
// Sortie attendue :
//   2
//   1
// ==================================================================

class Pile {
    private let items = [];

    func empiler(x) { self.items.add(x); }
    func depiler() { return self.items.pop(); }
    func taille() -> int { return self.items.size(); }
}

let p = new Pile();
p.empiler(1);
p.empiler(2);
println(p.depiler());
println(p.taille());
