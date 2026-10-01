// ==================================================================
// Exemple 591 — Éditeur de texte avec annuler / rétablir
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : deux piles d'états (undo et redo).
// ------------------------------------------------------------------
// Sortie attendue :
//   Bonjour
//   Bon
//   Bonjour
// ==================================================================

class Editeur {
    let texte: str = "";
    private let passe: List<str> = [];
    private let futur: List<str> = [];

    func ajouter(s: str) {
        self.passe.add(self.texte);
        self.futur.clear();
        self.texte = self.texte + s;
    }

    func annuler() {
        if !self.passe.is_empty() {
            self.futur.add(self.texte);
            self.texte = self.passe.pop();
        }
    }

    func retablir() {
        if !self.futur.is_empty() {
            self.passe.add(self.texte);
            self.texte = self.futur.pop();
        }
    }
}

let e = new Editeur();
e.ajouter("Bon");
e.ajouter("jour");
println(e.texte);
e.annuler();
println(e.texte);
e.retablir();
println(e.texte);
