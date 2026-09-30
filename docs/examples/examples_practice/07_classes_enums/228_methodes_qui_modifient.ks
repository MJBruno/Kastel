// ==================================================================
// Exemple 228 — Méthodes qui modifient l'objet
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// self désigne l'objet courant.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
// ==================================================================

class Compteur {
    let n: int = 0;

    func incrementer() {
        self.n += 1;
    }

    func valeur() -> int {
        return self.n;
    }
}

let c = new Compteur();
c.incrementer();
c.incrementer();
c.incrementer();
println(c.valeur());
