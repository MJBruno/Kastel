// ==================================================================
// Exemple 232 — Héritage
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// class Fille: Mere hérite des champs et méthodes de Mere.
// ------------------------------------------------------------------
// Sortie attendue :
//   Je suis animal
//   Miaou
// ==================================================================

class Animal {
    let nom: str = "animal";

    func se_presenter() -> str {
        return "Je suis " + self.nom;
    }
}

class Chat: Animal {
    func miauler() -> str {
        return "Miaou";
    }
}

let c = new Chat();
println(c.se_presenter());
println(c.miauler());
