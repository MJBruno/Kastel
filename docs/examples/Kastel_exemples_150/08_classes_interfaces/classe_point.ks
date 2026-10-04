// ====================================================================
// Kastel — Une première classe
// Notions : class, champs, initialize, méthodes, new, self
// Résultat attendu :
//   (3, 4)
//   (4, 6)
//   distance² = 52
// ====================================================================

class Point {
    let x: int;
    let y: int;

    func initialize(x: int, y: int) {
        self.x = x;
        self.y = y;
    }

    func decaler(dx: int, dy: int) {
        self.x = self.x + dx;
        self.y = self.y + dy;
    }

    func texte() -> str {
        return "(" + str(self.x) + ", " + str(self.y) + ")";
    }

    func distance_carree() -> int {
        return self.x * self.x + self.y * self.y;
    }
}

let p = new Point(3, 4);
println(p.texte());

p.decaler(1, 2);
println(p.texte());
println("distance² = {}", p.distance_carree());
