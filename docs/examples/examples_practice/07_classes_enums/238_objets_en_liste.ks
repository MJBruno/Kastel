// ==================================================================
// Exemple 238 — Liste d'objets
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Parcourir une collection d'objets et appeler leurs méthodes.
// ------------------------------------------------------------------
// Sortie attendue :
//   7
// ==================================================================

class Produit {
    let nom: str;
    let prix: int;

    func initialize(nom: str, prix: int) {
        self.nom = nom;
        self.prix = prix;
    }
}

let panier = [new Produit("stylo", 2), new Produit("cahier", 5)];
let total = 0;
for p in panier {
    total += p.prix;
}
println(total);
