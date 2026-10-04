// ====================================================================
// Kastel — Pile générique
// Notions : class Pile<T> avec une liste interne
// Résultat attendu :
//   taille = 3
//   sommet = 30
//   30
//   20
//   taille = 1
// ====================================================================

class Pile<T> {
    private let elements: List<T>;

    func initialize() {
        self.elements = [];
    }

    func empiler(valeur: T) {
        self.elements.add(valeur);
    }

    func depiler() -> T {
        return self.elements.pop();
    }

    func sommet() -> T {
        return self.elements.last();
    }

    func taille() -> int {
        return self.elements.size();
    }
}

let pile: Pile<int> = new Pile<int>();
pile.empiler(10);
pile.empiler(20);
pile.empiler(30);

println("taille = {}", pile.taille());
println("sommet = {}", pile.sommet());
println(pile.depiler());
println(pile.depiler());
println("taille = {}", pile.taille());
