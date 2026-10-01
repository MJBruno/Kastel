// ==================================================================
// Exemple 576 — Formes géométriques
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : une interface Forme et plusieurs classes ; la somme des aires ne connaît que l'interface.
// ------------------------------------------------------------------
// Sortie attendue :
//   carré : 9
//   rectangle : 10
//   triangle : 6
//   total : 25
// ==================================================================

interface Forme {
    func aire() -> int;
    func nom() -> str;
}

class Carre: Forme {
    let c: int;
    func initialize(c: int) { self.c = c; }
    func aire() -> int { return self.c * self.c; }
    func nom() -> str { return "carré"; }
}

class Rectangle: Forme {
    let l: int;
    let h: int;
    func initialize(l: int, h: int) { self.l = l; self.h = h; }
    func aire() -> int { return self.l * self.h; }
    func nom() -> str { return "rectangle"; }
}

class Triangle: Forme {
    let b: int;
    let h: int;
    func initialize(b: int, h: int) { self.b = b; self.h = h; }
    func aire() -> int { return idiv(self.b * self.h, 2); }
    func nom() -> str { return "triangle"; }
}

let formes = [new Carre(3), new Rectangle(2, 5), new Triangle(4, 3)];
let total = 0;
for f in formes {
    println("{} : {}", f.nom(), f.aire());
    total += f.aire();
}
println("total : {}", total);
