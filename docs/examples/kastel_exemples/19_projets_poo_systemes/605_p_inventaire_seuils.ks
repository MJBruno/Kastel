// ==================================================================
// Exemple 605 — Inventaire avec seuils
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : repérer les produits à réapprovisionner.
// ------------------------------------------------------------------
// Sortie attendue :
//   à réapprovisionner : cahier
//   à réapprovisionner : sac
// ==================================================================

class Produit {
    let nom: str;
    let quantite: int;
    let seuil: int;

    func initialize(nom: str, quantite: int, seuil: int) {
        self.nom = nom;
        self.quantite = quantite;
        self.seuil = seuil;
    }

    func a_commander() -> bool {
        return self.quantite < self.seuil;
    }
}

let stock = [new Produit("stylo", 50, 20), new Produit("cahier", 3, 10), new Produit("sac", 1, 2)];
for p in stock {
    if p.a_commander() {
        println("à réapprovisionner : " + p.nom);
    }
}
