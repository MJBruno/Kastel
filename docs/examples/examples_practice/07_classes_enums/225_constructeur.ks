// ==================================================================
// Exemple 225 — Le constructeur initialize
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// initialize(...) est appelé par new pour préparer l'objet.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ada a 36 ans
// ==================================================================

class Personne {
    let nom: str;
    let age: int;

    func initialize(nom: str, age: int) {
        self.nom = nom;
        self.age = age;
    }

    func presentation() -> str {
        return self.nom + " a " + str(self.age) + " ans";
    }
}

let p = new Personne("Ada", 36);
println(p.presentation());
