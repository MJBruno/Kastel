// ==================================================================
// Exemple 590 — Tampon circulaire
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : garder seulement les N derniers éléments.
// ------------------------------------------------------------------
// Sortie attendue :
//   [3, 4, 5]
// ==================================================================

class Tampon {
    private let capacite: int;
    private let items: List<int> = [];

    func initialize(c: int) { self.capacite = c; }

    func ajouter(x: int) {
        if self.items.size() == self.capacite {
            self.items.remove_at(0);
        }
        self.items.add(x);
    }

    func contenu() -> List<int> { return self.items.copy(); }
}

let t = new Tampon(3);
for i in range(1, 6) {
    t.ajouter(i);
}
println(t.contenu());
