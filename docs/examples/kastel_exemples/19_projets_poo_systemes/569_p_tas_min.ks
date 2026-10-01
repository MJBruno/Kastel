// ==================================================================
// Exemple 569 — Tas binaire (file de priorité)
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : un tas min stocké dans une liste ; extraire donne toujours le plus petit.
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 2, 3, 5, 8, 9]
// ==================================================================

class TasMin {
    private let d: List<int> = [];

    func ajouter(v: int) {
        self.d.add(v);
        let i = self.d.size() - 1;
        while i > 0 {
            let p = idiv(i - 1, 2);
            if self.d[p] <= self.d[i] { break; }
            let t = self.d[p];
            self.d[p] = self.d[i];
            self.d[i] = t;
            i = p;
        }
    }

    func extraire() -> int {
        let racine = self.d[0];
        let dernier = self.d.pop();
        if !self.d.is_empty() {
            self.d[0] = dernier;
            let i = 0;
            let n = self.d.size();
            while true {
                let g = 2 * i + 1;
                let dr = 2 * i + 2;
                let m = i;
                if g < n && self.d[g] < self.d[m] { m = g; }
                if dr < n && self.d[dr] < self.d[m] { m = dr; }
                if m == i { break; }
                let t = self.d[m];
                self.d[m] = self.d[i];
                self.d[i] = t;
                i = m;
            }
        }
        return racine;
    }

    func vide() -> bool { return self.d.is_empty(); }
}

let tas = new TasMin();
for v in [5, 3, 8, 1, 2, 9] {
    tas.ajouter(v);
}
let sortie = [];
while !tas.vide() {
    sortie.add(tas.extraire());
}
println(sortie);
