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

interface Animal {
    func se_presenter() -> str;
}

class Chat : Animal {
    let nom: str = "chat";

    func se_presenter() -> str {
        return "Je suis " + self.nom;
    }

    func miauler() -> str {
        return "Miaou";
    }
}

let c = new Chat();

println(c.se_presenter());
println(c.miauler());

