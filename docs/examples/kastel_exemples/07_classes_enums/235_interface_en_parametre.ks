// ==================================================================
// Exemple 235 — Interface comme type de paramètre
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Toute classe qui implémente l'interface est acceptée.
// ------------------------------------------------------------------
// Sortie attendue :
//   Bonjour R2
// ==================================================================

interface Nommable {
    func nom() -> str;
}

class Robot: Nommable {
    func nom() -> str { return "R2"; }
}

func saluer(x: Nommable) {
    println("Bonjour " + x.nom());
}

saluer(new Robot());
