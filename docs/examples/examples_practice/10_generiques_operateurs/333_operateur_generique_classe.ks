// ==================================================================
// Exemple 333 — Fonction générique avec une classe Add
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// La même fonction additionne des entiers ou des objets.
// ------------------------------------------------------------------
// Sortie attendue :
//   5
//   500
// ==================================================================

class Poids : Add {
    let g: int;

    func initialize(g: int) { self.g = g; }
    func add(o: Poids) -> Poids { return new Poids(self.g + o.g); }
}

func somme<T: Add>(a: T, b: T) -> T {
    return a + b;
}

println(somme(2, 3));
println(somme(new Poids(200), new Poids(300)).g);
