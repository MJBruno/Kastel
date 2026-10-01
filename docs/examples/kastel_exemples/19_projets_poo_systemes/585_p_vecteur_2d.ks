// ==================================================================
// Exemple 585 — Vecteurs 2D
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : addition de vecteurs avec l'opérateur + et produit scalaire.
// ------------------------------------------------------------------
// Sortie attendue :
//   (4, 6)
//   11
// ==================================================================

class Vec : Add {
    let x: int;
    let y: int;

    func initialize(x: int, y: int) { self.x = x; self.y = y; }

    func add(o: Vec) -> Vec { return new Vec(self.x + o.x, self.y + o.y); }

    func scalaire(o: Vec) -> int { return self.x * o.x + self.y * o.y; }
}

let v = new Vec(1, 2) + new Vec(3, 4);
println("({}, {})", v.x, v.y);
println(new Vec(1, 2).scalaire(new Vec(3, 4)));
