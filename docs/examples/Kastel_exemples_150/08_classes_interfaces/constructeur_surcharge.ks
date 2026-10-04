// ====================================================================
// Kastel — Constructeurs surchargés
// Notions : plusieurs initialize distingués par leur nombre d'arguments
// Résultat attendu :
//   0 0
//   5 5
//   3 4
// ====================================================================

class Point {
    let x: int = 0;
    let y: int = 0;

    func initialize() {}

    func initialize(valeur: int) {
        self.x = valeur;
        self.y = valeur;
    }

    func initialize(x: int, y: int) {
        self.x = x;
        self.y = y;
    }

    func texte() -> str {
        return str(self.x) + " " + str(self.y);
    }
}

println(new Point().texte());
println(new Point(5).texte());
println(new Point(3, 4).texte());
