// ====================================================================
// Kastel — Surcharge de == et de <
// Notions : Eq<T> avec equals, Ord<T> avec compare
// Résultat attendu :
//   true
//   false
//   true
//   true
//   false
// ====================================================================

class Mot: Eq<str> {
    let valeur: str;

    func initialize(valeur: str) { self.valeur = valeur; }

    func equals(other: str) -> bool {
        return self.valeur == other;
    }
}

class Version: Ord<int> {
    let numero: int;

    func initialize(numero: int) { self.numero = numero; }

    func compare(other: int) -> int {
        return self.numero - other;
    }
}

let mot = new Mot("kastel");
println(mot == "kastel");
println(mot == "autre");

let v = new Version(5);
println(v < 10);
println(v >= 3);
println(v > 5);
