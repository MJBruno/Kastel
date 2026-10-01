// ==================================================================
// Exemple 578 — Parking
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : des places numérotées, entrée et sortie de véhicules.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ok(0)
//   Ok(1)
//   Err(complet)
//   Ok(0)
// ==================================================================

class Parking {
    private let places: List<str> = [];

    func initialize(capacite: int) {
        for i in range(capacite) {
            self.places.add("");
        }
    }

    func entrer(plaque: str) -> Result<int, str> {
        for i in range(self.places.size()) {
            if self.places[i] == "" {
                self.places[i] = plaque;
                return Ok(i);
            }
        }
        return Err("complet");
    }

    func sortir(plaque: str) {
        let i = self.places.index_of(plaque);
        if i != -1 {
            self.places[i] = "";
        }
    }
}

let p = new Parking(2);
println(p.entrer("AA-111"));
println(p.entrer("BB-222"));
println(p.entrer("CC-333"));
p.sortir("AA-111");
println(p.entrer("CC-333"));
