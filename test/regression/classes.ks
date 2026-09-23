// À placer dans test/regression/classes.ks
// Couvre : champs typés, public/private, constructeur implicite,
// surcharge de initialize(), surcharge de méthode.

class Point {
    private let x: int = 0;
    private let y: int = 0;

    initialize() {}

    initialize(x: int, y: int) {
        this.x = x;
        this.y = y;
    }

    func sum() -> int {
        return this.x + this.y;
    }

    func combine(other: Point) -> int {
        return this.sum() + other.sum();
    }
}

let origin = Point();
let p = Point(3, 4);

print(origin.sum());     // 0
print(p.sum());          // 7
print(p.combine(origin)); // 7

class Empty {
    private let ready: bool = true;
}

// Constructeur par défaut implicite (aucun `initialize` déclaré).
let e = Empty();
print("ok");              // ok
