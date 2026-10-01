// ==================================================================
// Exemple 566 — File à double entrée
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : ajouter et retirer aux deux extrémités.
// ------------------------------------------------------------------
// Sortie attendue :
//   [0, 1, 2]
//   2
//   0
//   [1]
// ==================================================================

class Deque {
    private let items: List<int> = [];

    func ajouter_debut(x: int) { self.items.insert(0, x); }
    func ajouter_fin(x: int) { self.items.add(x); }
    func retirer_debut() -> int { return self.items.remove_at(0); }
    func retirer_fin() -> int { return self.items.pop(); }
    func contenu() -> List<int> { return self.items.copy(); }
}

let d = new Deque();
d.ajouter_fin(1);
d.ajouter_fin(2);
d.ajouter_debut(0);
println(d.contenu());
println(d.retirer_fin());
println(d.retirer_debut());
println(d.contenu());
