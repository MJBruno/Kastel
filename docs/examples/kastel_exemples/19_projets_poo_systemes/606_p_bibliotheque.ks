// ==================================================================
// Exemple 606 — Bibliothèque : emprunts
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : emprunter et rendre des livres, avec des erreurs explicites.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ok(emprunté : Dune)
//   Err(déjà emprunté)
//   Ok(emprunté : Dune)
//   Err(inconnu)
// ==================================================================

class Bibliotheque {
    private let disponible = {"Dune": true, "Emma": true};

    func emprunter(titre: str) -> Result<str, str> {
        if !self.disponible.contains(titre) { return Err("inconnu"); }
        if !self.disponible[titre] { return Err("déjà emprunté"); }
        self.disponible[titre] = false;
        return Ok("emprunté : " + titre);
    }

    func rendre(titre: str) {
        self.disponible[titre] = true;
    }
}

let b = new Bibliotheque();
println(b.emprunter("Dune"));
println(b.emprunter("Dune"));
b.rendre("Dune");
println(b.emprunter("Dune"));
println(b.emprunter("Zzz"));
