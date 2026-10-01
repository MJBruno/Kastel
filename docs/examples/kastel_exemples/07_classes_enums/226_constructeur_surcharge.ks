// ==================================================================
// Exemple 226 — Constructeur surchargé
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Plusieurs initialize avec des paramètres différents.
// ------------------------------------------------------------------
// Sortie attendue :
//   0
//   7
// ==================================================================

class Point {
    let x: int = 0;
    let y: int = 0;

    func initialize() {}

    func initialize(x: int, y: int) {
        self.x = x;
        self.y = y;
    }

    func somme() -> int {
        return self.x + self.y;
    }
}

let a = new Point();
let b = new Point(3, 4);
println(a.somme());
println(b.somme());
