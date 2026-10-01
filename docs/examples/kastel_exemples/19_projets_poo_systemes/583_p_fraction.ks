// ==================================================================
// Exemple 583 — Fractions exactes
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : une classe Fraction qui se simplifie toute seule et s'additionne avec +.
// ------------------------------------------------------------------
// Sortie attendue :
//   5/6
//   1/2
//   1/1
// ==================================================================

func pgcd(a: int, b: int) -> int {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    return a;
}

class Fraction : Add {
    let n: int;
    let d: int;

    func initialize(n: int, d: int) {
        let g = pgcd(abs(n), abs(d));
        self.n = idiv(n, g);
        self.d = idiv(d, g);
    }

    func add(o: Fraction) -> Fraction {
        return new Fraction(self.n * o.d + o.n * self.d, self.d * o.d);
    }

    func fois(o: Fraction) -> Fraction {
        return new Fraction(self.n * o.n, self.d * o.d);
    }

    func texte() -> str {
        return str(self.n) + "/" + str(self.d);
    }
}

println((new Fraction(1, 2) + new Fraction(1, 3)).texte());
println(new Fraction(2, 3).fois(new Fraction(3, 4)).texte());
println((new Fraction(1, 2) + new Fraction(1, 2)).texte());
