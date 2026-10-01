// ==================================================================
// Exemple 570 — Graphe non orienté
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : liste d'adjacence dans un dict et parcours en largeur.
// ------------------------------------------------------------------
// Sortie attendue :
//   ["A", "B", "C", "D", "E"]
// ==================================================================

class Graphe {
    private let adj = dict();

    func ajouter_arete(a: str, b: str) {
        if !self.adj.contains(a) { self.adj[a] = []; }
        if !self.adj.contains(b) { self.adj[b] = []; }
        self.adj[a].add(b);
        self.adj[b].add(a);
    }

    func largeur(depart: str) -> List<str> {
        let visites = Set(depart);
        let file = [depart];
        let ordre = [];
        while !file.is_empty() {
            let s = file.remove_at(0);
            ordre.add(s);
            for v in self.adj[s] {
                if !visites.contains(v) {
                    visites.add(v);
                    file.add(v);
                }
            }
        }
        return ordre;
    }
}

let g = new Graphe();
g.ajouter_arete("A", "B");
g.ajouter_arete("A", "C");
g.ajouter_arete("B", "D");
g.ajouter_arete("C", "D");
g.ajouter_arete("D", "E");
println(g.largeur("A"));
