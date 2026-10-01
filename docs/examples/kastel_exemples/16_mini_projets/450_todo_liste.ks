// ==================================================================
// Exemple 450 — Liste de tâches
// Catégorie : Mini-projets
// ------------------------------------------------------------------
// Une classe qui gère des tâches avec un état terminé / à faire.
// ------------------------------------------------------------------
// Sortie attendue :
//   [x] écrire du code
//   [ ] tester
// ==================================================================

class Tache {
    let titre: str;
    let faite: bool = false;

    func initialize(titre: str) {
        self.titre = titre;
    }
}

class ListeTaches {
    private let taches = [];

    func ajouter(titre: str) {
        self.taches.add(new Tache(titre));
    }

    func terminer(i: int) {
        self.taches[i].faite = true;
    }

    func afficher() {
        for t in self.taches {
            let marque = t.faite ? "[x]" : "[ ]";
            println(marque + " " + t.titre);
        }
    }
}

let l = new ListeTaches();
l.ajouter("écrire du code");
l.ajouter("tester");
l.terminer(0);
l.afficher();
