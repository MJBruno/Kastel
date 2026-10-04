// ====================================================================
// Kastel — Classe générique
// Notions : class Boite<T>, inférence du type à la construction
// Résultat attendu :
//   42
//   kastel
// ====================================================================

class Boite<T> {
    private let contenu: T;

    func initialize(contenu: T) {
        self.contenu = contenu;
    }

    func lire() -> T {
        return self.contenu;
    }
}

let a: Boite<int> = new Boite(42);
let b: Boite<str> = new Boite<str>("kastel");

println(a.lire());
println(b.lire());
