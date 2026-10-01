// ==================================================================
// Exemple 233 — Redéfinir une méthode
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// La classe dérivée peut fournir sa propre version d'une méthode.
// ------------------------------------------------------------------
// Sortie attendue :
//   forme
//   cercle
// ==================================================================

class Forme {
    func nom() -> str {
        return "forme";
    }
}

class Cercle: Forme {
    func nom() -> str {
        return "cercle";
    }
}

println(new Forme().nom());
println(new Cercle().nom());
