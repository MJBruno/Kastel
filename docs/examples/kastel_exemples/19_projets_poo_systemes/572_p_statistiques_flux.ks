// ==================================================================
// Exemple 572 — Statistiques au fil de l'eau
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : moyenne, minimum et maximum mis à jour à chaque valeur.
// ------------------------------------------------------------------
// Sortie attendue :
//   4.0 2.0 6.0
// ==================================================================

class Stats {
    private let n: int = 0;
    private let somme: float = 0.0;
    private let mini: float = 0.0;
    private let maxi: float = 0.0;

    func ajouter(x: float) {
        if self.n == 0 {
            self.mini = x;
            self.maxi = x;
        } else {
            self.mini = min(self.mini, x);
            self.maxi = max(self.maxi, x);
        }
        self.n = self.n + 1;
        self.somme = self.somme + x;
    }

    func moyenne() -> float { return self.somme / self.n; }
    func minimum() -> float { return self.mini; }
    func maximum() -> float { return self.maxi; }
}

let s = new Stats();
for x in [2.0, 4.0, 6.0] {
    s.ajouter(x);
}
println("{:.1f} {:.1f} {:.1f}", s.moyenne(), s.minimum(), s.maximum());
