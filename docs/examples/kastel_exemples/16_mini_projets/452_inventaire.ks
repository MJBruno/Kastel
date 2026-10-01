// ==================================================================
// Exemple 452 — Inventaire de magasin
// Catégorie : Mini-projets
// ------------------------------------------------------------------
// Un dict qui associe un produit à sa quantité.
// ------------------------------------------------------------------
// Sortie attendue :
//   15
//   false
//   true
//   12
// ==================================================================

let stock = dict();

func ajouter(produit: str, quantite: int) {
    stock[produit] = stock.get_or(produit, 0) + quantite;
}

func vendre(produit: str, quantite: int) -> bool {
    if stock.get_or(produit, 0) < quantite {
        return false;
    }
    stock[produit] = stock[produit] - quantite;
    return true;
}

ajouter("pommes", 10);
ajouter("poires", 4);
ajouter("pommes", 5);
println(stock["pommes"]);
println(vendre("poires", 10));
println(vendre("pommes", 3));
println(stock["pommes"]);
