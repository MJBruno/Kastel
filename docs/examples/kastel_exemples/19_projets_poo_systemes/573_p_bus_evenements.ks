// ==================================================================
// Exemple 573 — Bus d'événements
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : abonner des fonctions à un nom d'événement puis les notifier.
// ------------------------------------------------------------------
// Sortie attendue :
//   1:salut
//   2:salut
// ==================================================================

class Bus {
    private let abonnes = dict();

    func sur(evenement: str, f) {
        if !self.abonnes.contains(evenement) {
            self.abonnes[evenement] = [];
        }
        self.abonnes[evenement].add(f);
    }

    func emettre(evenement: str, donnee) {
        for f in self.abonnes.get_or(evenement, []) {
            f(donnee);
        }
    }
}

let bus = new Bus();
bus.sur("msg", x => println("1:" + x));
bus.sur("msg", x => println("2:" + x));
bus.emettre("msg", "salut");
bus.emettre("autre", "ignoré");
