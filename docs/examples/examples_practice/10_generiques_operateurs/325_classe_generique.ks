// ==================================================================
// Exemple 325 — Classe générique
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// Une boîte qui garde une valeur de type T.
// ------------------------------------------------------------------
// Sortie attendue :
//   42
//   kastel
// ==================================================================

class Boite<T> {
    let valeur: T;

    func initialize(v: T) {
        self.valeur = v;
    }

    func lire() -> T {
        return self.valeur;
    }
}

let a = new Boite<int>(42);
let b = new Boite<str>("kastel");
println(a.lire());
println(b.lire());
