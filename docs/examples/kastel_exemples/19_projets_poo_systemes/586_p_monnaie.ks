// ==================================================================
// Exemple 586 — Type Monnaie
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : des montants en centimes qui s'additionnent, affichés en euros.
// ------------------------------------------------------------------
// Sortie attendue :
//   7.80 EUR
// ==================================================================

class Euros : Add {
    let centimes: int;

    func initialize(c: int) { self.centimes = c; }

    func add(o: Euros) -> Euros {
        return new Euros(self.centimes + o.centimes);
    }

    func texte() -> str {
        return format("{}.{:02d} EUR", idiv(self.centimes, 100), self.centimes % 100);
    }
}

let total = new Euros(550) + new Euros(225) + new Euros(5);
println(total.texte());
