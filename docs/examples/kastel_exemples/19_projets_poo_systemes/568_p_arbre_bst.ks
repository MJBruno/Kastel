// ==================================================================
// Exemple 568 — Arbre binaire de recherche
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : insertion récursive et parcours dans l'ordre (résultat trié).
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 3, 4, 5, 7, 8, 9]
// ==================================================================

class Noeud {
    let valeur: int;
    let gauche: Option<Noeud> = None;
    let droite: Option<Noeud> = None;

    func initialize(v: int) { self.valeur = v; }

    func inserer(v: int) {
        if v < self.valeur {
            match self.gauche {
                Some(n) => { n.inserer(v); }
                None => { self.gauche = Some(new Noeud(v)); }
            }
        } else {
            match self.droite {
                Some(n) => { n.inserer(v); }
                None => { self.droite = Some(new Noeud(v)); }
            }
        }
    }

    func parcourir(res: List<int>) {
        match self.gauche {
            Some(n) => { n.parcourir(res); }
            None => {}
        }
        res.add(self.valeur);
        match self.droite {
            Some(n) => { n.parcourir(res); }
            None => {}
        }
    }
}

let racine = new Noeud(5);
for v in [3, 8, 1, 4, 9, 7] {
    racine.inserer(v);
}
let res = [];
racine.parcourir(res);
println(res);
