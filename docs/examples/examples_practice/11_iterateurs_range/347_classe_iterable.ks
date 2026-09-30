// ==================================================================
// Exemple 347 — Une classe itérable
// Catégorie : Itérateurs et range
// ------------------------------------------------------------------
// Une classe qui fournit iter() peut être parcourue par un for.
// ------------------------------------------------------------------
// Sortie attendue :
//   x
//   y
// ==================================================================

class Sac {
    private let elements = [];

    func ajouter(x) { self.elements.add(x); }
    func iter() { return self.elements.iter(); }
}

let s = new Sac();
s.ajouter("x");
s.ajouter("y");
for e in s {
    println(e);
}
