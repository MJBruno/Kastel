// ==================================================================
// Exemple 574 — Observateur : alerte de température
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : un capteur prévient ses observateurs à chaque mesure.
// ------------------------------------------------------------------
// Sortie attendue :
//   alerte : 30
//   2 mesures
// ==================================================================

class Capteur {
    private let observateurs = [];
    let mesures: int = 0;

    func observer(f) { self.observateurs.add(f); }

    func mesurer(t: int) {
        self.mesures = self.mesures + 1;
        for o in self.observateurs {
            o(t);
        }
    }
}

let c = new Capteur();
c.observer(t => { if t > 25 { println("alerte : " + str(t)); } });
c.mesurer(20);
c.mesurer(30);
println("{} mesures", c.mesures);
