// ==================================================================
// Exemple 459 — Petit cache
// Catégorie : Mini-projets
// ------------------------------------------------------------------
// Un dict et une limite de taille.
// ------------------------------------------------------------------
// Sortie attendue :
//   None
//   Some(3)
// ==================================================================

class Cache {
    private let donnees = dict();
    private let ordre = [];
    private let capacite: int = 2;

    func mettre(cle: str, valeur: int) {
        if !self.donnees.contains(cle) {
            if self.ordre.size() == self.capacite {
                let ancienne = self.ordre.remove_at(0);
                self.donnees.remove(ancienne);
            }
            self.ordre.add(cle);
        }
        self.donnees[cle] = valeur;
    }

    func lire(cle: str) -> Option<int> {
        if self.donnees.contains(cle) {
            return Some(self.donnees[cle]);
        }
        return None;
    }
}

let c = new Cache();
c.mettre("a", 1);
c.mettre("b", 2);
c.mettre("c", 3);       // évince "a"
println(c.lire("a"));
println(c.lire("c"));
