// ==================================================================
// Exemple 610 — Classe Matrice
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : produit de matrices carrées encapsulé dans une classe.
// ------------------------------------------------------------------
// Sortie attendue :
//   [[19, 22], [43, 50]]
// ==================================================================

class Matrice {
    let m: List<List<int>>;

    func initialize(m: List<List<int>>) { self.m = m; }

    func fois(o: Matrice) -> Matrice {
        let n = self.m.size();
        let res = [];
        for i in range(n) {
            let ligne = [];
            for j in range(n) {
                let s = 0;
                for k in range(n) {
                    s += self.m[i][k] * o.m[k][j];
                }
                ligne.add(s);
            }
            res.add(ligne);
        }
        return new Matrice(res);
    }
}

let a = new Matrice([[1, 2], [3, 4]]);
let b = new Matrice([[5, 6], [7, 8]]);
println(a.fois(b).m);
