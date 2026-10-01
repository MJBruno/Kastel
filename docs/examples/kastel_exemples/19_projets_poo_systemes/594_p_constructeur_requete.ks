// ==================================================================
// Exemple 594 — Patron Builder : requête SQL
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : construire une requête par appels chaînés.
// ------------------------------------------------------------------
// Sortie attendue :
//   SELECT nom, age FROM users WHERE age > 18 AND actif = 1
// ==================================================================

class Requete {
    private let champs: List<str> = [];
    private let table: str = "";
    private let conditions: List<str> = [];

    func selectionner(c: str) -> Requete { self.champs.add(c); return self; }
    func depuis(t: str) -> Requete { self.table = t; return self; }
    func ou_bien(c: str) -> Requete { self.conditions.add(c); return self; }

    func construire() -> str {
        let s = "SELECT " + ", ".join(self.champs) + " FROM " + self.table;
        if !self.conditions.is_empty() {
            s += " WHERE " + " AND ".join(self.conditions);
        }
        return s;
    }
}

let r = new Requete()
    .selectionner("nom")
    .selectionner("age")
    .depuis("users")
    .ou_bien("age > 18")
    .ou_bien("actif = 1");
println(r.construire());
