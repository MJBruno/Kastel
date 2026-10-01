// ==================================================================
// Exemple 589 — Ensemble de bits
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : stocker des drapeaux dans un seul entier.
// ------------------------------------------------------------------
// Sortie attendue :
//   42
//   true
//   false
//   2
// ==================================================================

class BitSet {
    private let valeur: int = 0;

    func activer(i: int) { self.valeur = self.valeur | (1 << i); }
    func desactiver(i: int) { self.valeur = self.valeur & ~(1 << i); }
    func est_actif(i: int) -> bool { return ((self.valeur >> i) & 1) == 1; }
    func brut() -> int { return self.valeur; }

    func compter() -> int {
        let n = 0;
        let v = self.valeur;
        while v > 0 {
            n += v & 1;
            v = v >> 1;
        }
        return n;
    }
}

let b = new BitSet();
b.activer(1);
b.activer(3);
b.activer(5);
println(b.brut());
println(b.est_actif(3));
println(b.est_actif(4));
b.desactiver(3);
println(b.compter());
