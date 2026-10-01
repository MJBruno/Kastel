// ==================================================================
// Exemple 224 — Une première classe
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Une classe regroupe des champs (données) et des méthodes (comportements).
// ------------------------------------------------------------------
// Sortie attendue :
//   Rex dit : Ouaf !
// ==================================================================

class Chien {
    let nom: str = "Rex";

    func aboyer() -> str {
        return self.nom + " dit : Ouaf !";
    }
}

let c = new Chien();
println(c.aboyer());
