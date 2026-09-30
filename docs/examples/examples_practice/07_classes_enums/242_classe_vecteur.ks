// ==================================================================
// Exemple 242 — Vecteur 2D
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Méthodes qui construisent de nouveaux objets.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
//   25
// ==================================================================

class Vec {
    let x: int;
    let y: int;

    func initialize(x: int, y: int) {
        self.x = x;
        self.y = y;
    }

    func plus(autre: Vec) -> Vec {
        return new Vec(self.x + autre.x, self.y + autre.y);
    }

    func norme_carree() -> int {
        return self.x * self.x + self.y * self.y;
    }
}

let v = new Vec(1, 2).plus(new Vec(2, 2));
println(v.x);
println(v.norme_carree());
