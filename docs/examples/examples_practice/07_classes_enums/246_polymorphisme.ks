// ==================================================================
// Exemple 246 — Polymorphisme par interface
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Une même boucle, des comportements différents selon la classe.
// ------------------------------------------------------------------
// Sortie attendue :
//   9
//   10
// ==================================================================

interface Forme {
    func aire() -> int;
}

class Carre: Forme {
    let c: int;
    func initialize(c: int) { self.c = c; }
    func aire() -> int { return self.c * self.c; }
}

class Rectangle: Forme {
    let l: int;
    let h: int;
    func initialize(l: int, h: int) { self.l = l; self.h = h; }
    func aire() -> int { return self.l * self.h; }
}

let formes = [new Carre(3), new Rectangle(2, 5)];
for f in formes {
    println(f.aire());
}
