// ==================================================================
// Exemple 330 — Surcharger * avec Mul
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// Mul -> méthode mul.
// ------------------------------------------------------------------
// Sortie attendue :
//   42
// ==================================================================

class Matrice2 : Mul {
    let v: int;

    func initialize(v: int) { self.v = v; }

    func mul(o: Matrice2) -> Matrice2 { return new Matrice2(self.v * o.v); }
}

println((new Matrice2(6) * new Matrice2(7)).v);
