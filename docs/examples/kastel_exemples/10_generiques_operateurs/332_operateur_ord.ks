// ==================================================================
// Exemple 332 — Surcharger < avec Ord
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// compare renvoie un entier négatif, nul ou positif.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

class Version : Ord {
    let n: int;

    func initialize(n: int) { self.n = n; }

    func compare(autre: Version) -> int {
        return self.n - autre.n;
    }
}

let a = new Version(1);
let b = new Version(2);
println(a < b);
println(a > b);
