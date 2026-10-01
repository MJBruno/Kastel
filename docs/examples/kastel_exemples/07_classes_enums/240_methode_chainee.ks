// ==================================================================
// Exemple 240 — Chaîner les appels
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Une méthode qui renvoie self permet d'enchaîner.
// ------------------------------------------------------------------
// Sortie attendue :
//   Kastel
// ==================================================================

class Constructeur {
    let texte: str = "";

    func ajouter(s: str) -> Constructeur {
        self.texte += s;
        return self;
    }
}

let r = new Constructeur().ajouter("Ka").ajouter("st").ajouter("el");
println(r.texte);
