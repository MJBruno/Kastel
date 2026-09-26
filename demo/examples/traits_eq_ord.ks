// traits_eq_ord.ks
//
// Eq<Rhs> et Ord<Rhs> suivent le meme principe que PartialEq<Rhs> /
// PartialOrd<Rhs> en Rust : deux methodes conventionnelles, "equals" et
// "compare", avec une sortie FIXEE par le contrat (bool pour Eq, int pour
// Ord), quel que soit Rhs. Le compilateur traduit ensuite chaque
// operateur de comparaison vers le bon appel :
//   a == b   -> a.equals(b)
//   a < b    -> a.compare(b) < 0
//   a >= b   -> !(a.compare(b) < 0)

class Version : Eq<Version>, Ord<Version> {
    public let majeur: int;
    public let mineur: int;

    func initialize(majeur: int, mineur: int) {
        this.majeur = majeur;
        this.mineur = mineur;
    }

    func equals(other: Version) -> bool {
        return this.majeur == other.majeur && this.mineur == other.mineur;
    }

    // "compare" renvoie -1, 0 ou 1 (ordre a trois valeurs, comme
    // std::cmp::Ordering en Rust) ; <, <=, >, >= restent bien de type bool.
    func compare(other: Version) -> int {
        if (this.majeur != other.majeur) {
            return this.majeur - other.majeur;
        }
        return this.mineur - other.mineur;
    }
}

let v1 = new Version(1, 4);
let v2 = new Version(1, 9);

print(v1 == v2); // false : passe par equals()
print(v1 < v2);  // true  : passe par compare()
print(v1 >= v2); // false

// Rhs peut differer de Self, comme PartialEq<str> pour un type "maison".
class Etiquette : Eq<str> {
    public let valeur: str;

    func initialize(valeur: str) {
        this.valeur = valeur;
    }

    func equals(other: str) -> bool {
        return this.valeur == other;
    }
}

let e = new Etiquette("urgent");
print(e == "urgent"); // true, compare directement a une str
