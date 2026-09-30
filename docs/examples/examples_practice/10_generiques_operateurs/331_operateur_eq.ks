// ==================================================================
// Exemple 331 — Surcharger == avec Eq
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// La méthode equals définit ce que « égal » veut dire.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

class Point : Eq {
    let x: int;
    let y: int;

    func initialize(x: int, y: int) { self.x = x; self.y = y; }

    func equals(autre: Point) -> bool {
        return self.x == autre.x && self.y == autre.y;
    }
}

println(new Point(1, 2) == new Point(1, 2));
println(new Point(1, 2) == new Point(3, 4));
