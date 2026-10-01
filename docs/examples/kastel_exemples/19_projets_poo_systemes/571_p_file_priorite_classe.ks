// ==================================================================
// Exemple 571 — File de priorité
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : les tâches urgentes (petit numéro) sortent en premier.
// ------------------------------------------------------------------
// Sortie attendue :
//   urgente
//   normale
//   basse
// ==================================================================

class FilePriorite {
    private let elements = [];

    func ajouter(priorite: int, nom: str) {
        let i = 0;
        while i < self.elements.size() && self.elements[i][0] <= priorite {
            i += 1;
        }
        self.elements.insert(i, (priorite, nom));
    }

    func suivant() -> str {
        return self.elements.remove_at(0)[1];
    }
}

let f = new FilePriorite();
f.ajouter(2, "normale");
f.ajouter(1, "urgente");
f.ajouter(3, "basse");
println(f.suivant());
println(f.suivant());
println(f.suivant());
