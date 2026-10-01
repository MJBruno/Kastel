// ==================================================================
// Exemple 231 — Membre protected
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// protected : accessible dans la classe et ses classes dérivées.
// ------------------------------------------------------------------
// Sortie attendue :
//   14
// ==================================================================

class Base {
    protected let valeur: int = 7;
}

class Derivee: Base {
    func lire() -> int {
        return self.valeur * 2;
    }
}

println(new Derivee().lire());
