// ==================================================================
// Exemple 602 — Mini tableur
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : des cellules qui contiennent un nombre ou une formule évaluée à la demande.
// ------------------------------------------------------------------
// Sortie attendue :
//   30
//   60
// ==================================================================

class Tableur {
    private let valeurs = dict();
    private let formules = dict();

    func poser(nom: str, v: int) { self.valeurs[nom] = v; }
    func formule(nom: str, f) { self.formules[nom] = f; }

    func valeur(nom: str) -> int {
        if self.formules.contains(nom) {
            let f = self.formules[nom];
            return f(self);
        }
        return self.valeurs[nom];
    }
}

let t = new Tableur();
t.poser("A1", 10);
t.poser("A2", 20);
t.formule("A3", s => s.valeur("A1") + s.valeur("A2"));
t.formule("A4", s => s.valeur("A3") * 2);
println(t.valeur("A3"));
println(t.valeur("A4"));
