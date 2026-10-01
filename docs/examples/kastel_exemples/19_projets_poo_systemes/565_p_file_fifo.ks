// ==================================================================
// Exemple 565 — File d'attente (FIFO)
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : enfiler à la fin, défiler au début.
// ------------------------------------------------------------------
// Sortie attendue :
//   Some(1)
//   Some(2)
//   1
// ==================================================================

class File {
    private let items: List<int> = [];

    func enfiler(x: int) { self.items.add(x); }

    func defiler() -> Option<int> {
        if self.items.is_empty() { return None; }
        return Some(self.items.remove_at(0));
    }

    func taille() -> int { return self.items.size(); }
}

let f = new File();
f.enfiler(1);
f.enfiler(2);
f.enfiler(3);
println(f.defiler());
println(f.defiler());
println(f.taille());
