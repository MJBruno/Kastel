// ==================================================================
// Exemple 329 — Surcharger - avec Sub
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// Sub -> méthode sub.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
//   5
// ==================================================================

class V2 : Sub {
    let x: int;
    let y: int;

    func initialize(x: int, y: int) { self.x = x; self.y = y; }

    func sub(o: V2) -> V2 { return new V2(self.x - o.x, self.y - o.y); }
}

let d = new V2(5, 8) - new V2(2, 3);
println(d.x);
println(d.y);
