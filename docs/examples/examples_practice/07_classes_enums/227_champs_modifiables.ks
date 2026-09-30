// ==================================================================
// Exemple 227 — Modifier les champs d'un objet
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Les champs publics se lisent et s'écrivent avec un point.
// ------------------------------------------------------------------
// Sortie attendue :
//   150
// ==================================================================

class Compte {
    let solde: int = 0;
}

let c = new Compte();
c.solde = 100;
c.solde += 50;
println(c.solde);
