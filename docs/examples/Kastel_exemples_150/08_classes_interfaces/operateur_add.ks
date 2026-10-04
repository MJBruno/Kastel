// ====================================================================
// Kastel — Surcharge de l'opérateur +
// Notions : interface Addable avec Self, a + b sur des objets
// Résultat attendu :
//   150 EUR
//   80 EUR
// ====================================================================

interface Addable {
    func add(other: Self) -> Self;
}

class Argent: Addable {
    let montant: int;

    func initialize(montant: int) {
        self.montant = montant;
    }

    func add(other: Self) -> Self {
        return new Argent(self.montant + other.montant);
    }

    func texte() -> str {
        return str(self.montant) + " EUR";
    }
}

let a = new Argent(100);
let b = new Argent(50);
let somme: Argent = a + b;
println(somme.texte());

let c = new Argent(30) + new Argent(50);
println(c.texte());
