// ==================================================================
// Exemple 334 — Opérateur hétérogène
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// Le type de droite et le type du résultat peuvent différer de la classe.
// ------------------------------------------------------------------
// Sortie attendue :
//   5.0
// ==================================================================

class Duree {
    let secondes: int;
    func initialize(s: int) { self.secondes = s; }
}

class Distance : Div<Duree, float> {
    let metres: int;
    func initialize(m: int) { self.metres = m; }
    func div(d: Duree) -> float { return self.metres / d.secondes; }
}

let vitesse = new Distance(100) / new Duree(20);
println(vitesse);
