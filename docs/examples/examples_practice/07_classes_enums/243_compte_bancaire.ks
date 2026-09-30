// ==================================================================
// Exemple 243 — Compte bancaire avec erreurs
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Une méthode qui lève une exception si le solde est insuffisant.
// ------------------------------------------------------------------
// Sortie attendue :
//   solde insuffisant
//   70
// ==================================================================

class Compte {
    private let solde: int = 100;

    func retirer(m: int) {
        if m > self.solde {
            throw "solde insuffisant";
        }
        self.solde -= m;
    }

    func solde_actuel() -> int { return self.solde; }
}

let c = new Compte();
c.retirer(30);
try {
    c.retirer(500);
} catch (e) {
    println(e);
}
println(c.solde_actuel());
