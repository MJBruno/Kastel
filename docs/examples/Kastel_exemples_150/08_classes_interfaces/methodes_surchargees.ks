// ====================================================================
// Kastel — Méthodes surchargées
// Notions : même nom, nombre d'arguments différent
// Résultat attendu :
//   3
//   7
//   12
// ====================================================================

class Calcul {
    func somme(a: int) -> int { return a; }
    func somme(a: int, b: int) -> int { return a + b; }
    func somme(a: int, b: int, c: int) -> int { return a + b + c; }
}

let c = new Calcul();
println(c.somme(3));
println(c.somme(3, 4));
println(c.somme(3, 4, 5));
