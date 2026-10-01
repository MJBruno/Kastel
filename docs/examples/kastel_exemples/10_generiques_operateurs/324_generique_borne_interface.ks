// ==================================================================
// Exemple 324 — Borne par interface
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// T: Interface : on peut appeler les méthodes de l'interface.
// ------------------------------------------------------------------
// Sortie attendue :
//   [livre]
// ==================================================================

interface Affichable {
    func rendre() -> str;
}

class Livre: Affichable {
    func rendre() -> str { return "livre"; }
}

func montrer<T: Affichable>(x: T) -> str {
    return "[" + x.rendre() + "]";
}

println(montrer(new Livre()));
