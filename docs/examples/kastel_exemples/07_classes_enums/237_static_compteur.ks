// ==================================================================
// Exemple 237 — Compter les instances
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Un champ static partagé par toutes les instances.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
// ==================================================================

class Ticket {
    static let total: int = 0;

    func initialize() {
        Ticket.total = Ticket.total + 1;
    }
}

new Ticket();
new Ticket();
new Ticket();
println(Ticket.total);
