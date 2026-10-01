// ==================================================================
// Exemple 244 — Classe d'erreur personnalisée
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// On peut lancer un objet : il porte les informations de l'erreur.
// ------------------------------------------------------------------
// Sortie attendue :
//   400 : âge négatif
// ==================================================================

class ErreurMetier {
    let code: int;
    let message: str;

    func initialize(code: int, message: str) {
        self.code = code;
        self.message = message;
    }
}

func verifier(age: int) {
    if age < 0 {
        throw new ErreurMetier(400, "âge négatif");
    }
}

try {
    verifier(-1);
} catch (e) {
    println("{} : {}", e.code, e.message);
}
