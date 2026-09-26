// traits_arithmetic.ks
//
// Chaque operateur (+, -, *, /, ...) correspond a un contrat intrinseque
// (Add<Rhs, Output>, Sub<Rhs, Output>, Div<Rhs, Output>, ...), exactement
// comme std::ops::Add<Rhs> en Rust. Une classe doit declarer explicitement
// le contrat dans sa liste de bases pour pouvoir utiliser l'operateur :
// une simple methode "add" ne suffit pas si "Add" n'est pas dans les bases.

class Argent : Add, Sub {
    public let centimes: int;

    func initialize(centimes: int) {
        this.centimes = centimes;
    }

    // Forme homogene : Rhs = Output = Self (comme `impl Add for Argent`).
    func add(other: Argent) -> Argent {
        return new Argent(this.centimes + other.centimes);
    }

    func sub(other: Argent) -> Argent {
        return new Argent(this.centimes - other.centimes);
    }

    func to_string() -> str {
        return str(this.centimes / 100.0) + " EUR";
    }
}

let prix: Argent = new Argent(1050);
let remise: Argent = new Argent(200);

let total: Argent = prix - remise + new Argent(50);
println(total.to_string());

// Contrat heterogene : Rhs et Output different du type receveur, comme
// `impl Div<i32> for Ratio { type Output = f64; }` en Rust.
class Ratio : Div<int, float> {
    public let numerateur: int;
    public let denominateur: int;

    func initialize(numerateur: int, denominateur: int) {
        this.numerateur = numerateur;
        this.denominateur = denominateur;
    }

    func div(other: int) -> float {
        return (this.numerateur / this.denominateur) / other;
    }
}

let r = new Ratio(3, 4);
let resultat: float = r / 2; // -> float, pas Ratio : le type est celui d'Output

println(resultat);
