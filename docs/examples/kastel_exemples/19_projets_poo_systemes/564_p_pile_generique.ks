// ==================================================================
// Exemple 564 — Pile générique
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : une pile Pile<T> qui marche pour tous les types, avec Option pour le cas vide.
// ------------------------------------------------------------------
// Sortie attendue :
//   Some(3)
//   Some(2)
//   1
//   Some(1)
//   None
// ==================================================================

class Pile<T> {
    private let items = [];

    func empiler(x: T) {
        self.items.add(x);
    }

    func depiler() -> Option<T> {
        if self.items.is_empty() {
            return None;
        }
        return Some(self.items.pop());
    }

    func taille() -> int {
        return self.items.size();
    }
}

let p = new Pile<int>();
p.empiler(1);
p.empiler(2);
p.empiler(3);
println(p.depiler());
println(p.depiler());
println(p.taille());
println(p.depiler());
println(p.depiler());
