// ==================================================================
// Exemple 567 — Liste chaînée
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : des noeuds reliés par un champ Option<Noeud>.
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 2, 3]
// ==================================================================

class Noeud {
    let valeur: int;
    let suivant: Option<Noeud> = None;

    func initialize(v: int) {
        self.valeur = v;
    }
}

class ListeChainee {
    private let tete: Option<Noeud> = None;

    func ajouter(v: int) {
        let nouveau = new Noeud(v);
        match self.tete {
            None => { self.tete = Some(nouveau); }
            Some(premier) => {
                let courant = premier;
                let fini = false;
                while !fini {
                    match courant.suivant {
                        Some(s) => { courant = s; }
                        None => {
                            courant.suivant = Some(nouveau);
                            fini = true;
                        }
                    }
                }
            }
        }
    }

    func vers_liste() -> List<int> {
        let res = [];
        match self.tete {
            None => { return res; }
            Some(premier) => {
                let courant = premier;
                res.add(courant.valeur);
                let fini = false;
                while !fini {
                    match courant.suivant {
                        Some(s) => { courant = s; res.add(s.valeur); }
                        None => { fini = true; }
                    }
                }
            }
        }
        return res;
    }
}

let l = new ListeChainee();
l.ajouter(1);
l.ajouter(2);
l.ajouter(3);
println(l.vers_liste());
