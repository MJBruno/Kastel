// ====================================================================
// Kastel — Interfaces
// Notions : interface, implémentation, liste polymorphe
// Résultat attendu :
//   Carré : 16.0
//   Rectangle : 12.0
//   Total : 28.0
// ====================================================================

interface Forme {
    func aire() -> float;
    func nom() -> str;
}

class Carre: Forme {
    let cote: float;

    func initialize(cote: float) { self.cote = cote; }
    func aire() -> float { return self.cote * self.cote; }
    func nom() -> str { return "Carré"; }
}

class Rectangle: Forme {
    let largeur: float;
    let hauteur: float;

    func initialize(largeur: float, hauteur: float) {
        self.largeur = largeur;
        self.hauteur = hauteur;
    }
    func aire() -> float { return self.largeur * self.hauteur; }
    func nom() -> str { return "Rectangle"; }
}

let formes: List<Forme> = [new Carre(4.0), new Rectangle(3.0, 4.0)];
let total = 0.0;

for f in formes {
    println("{} : {}", f.nom(), f.aire());
    total += f.aire();
}

println("Total : {}", total);
