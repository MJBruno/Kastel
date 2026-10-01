// ==================================================================
// Exemple 587 — Polynômes
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : évaluation par la méthode de Horner et dérivée.
// ------------------------------------------------------------------
// Sortie attendue :
//   15
//   [4, 3]
// ==================================================================

class Polynome {
    let coef: List<int>;      // du plus haut degré au plus bas

    func initialize(c: List<int>) { self.coef = c; }

    func evaluer(x: int) -> int {
        let r = 0;
        for c in self.coef {
            r = r * x + c;
        }
        return r;
    }

    func derivee() -> List<int> {
        let n = self.coef.size() - 1;
        let res = [];
        for i in range(n) {
            res.add(self.coef[i] * (n - i));
        }
        return res;
    }
}

let p = new Polynome([2, 3, 1]);        // 2x² + 3x + 1
println(p.evaluer(2));
println(p.derivee());
