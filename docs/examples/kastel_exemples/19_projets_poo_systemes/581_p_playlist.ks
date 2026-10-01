// ==================================================================
// Exemple 581 — Lecteur de playlist
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : morceau courant, suivant avec retour au début, durée totale.
// ------------------------------------------------------------------
// Sortie attendue :
//   B
//   C
//   A
//   10:25
// ==================================================================

class Playlist {
    private let titres = [];
    private let courant: int = 0;

    func ajouter(nom: str, secondes: int) {
        self.titres.add((nom, secondes));
    }

    func suivant() -> str {
        self.courant = (self.courant + 1) % self.titres.size();
        return self.titres[self.courant][0];
    }

    func duree_totale() -> str {
        let total = 0;
        for t in self.titres { total += t[1]; }
        return format("{}:{:02d}", idiv(total, 60), total % 60);
    }
}

let p = new Playlist();
p.ajouter("A", 200);
p.ajouter("B", 185);
p.ajouter("C", 240);
println(p.suivant());
println(p.suivant());
println(p.suivant());
println(p.duree_totale());
