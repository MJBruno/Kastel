// ==================================================================
// Exemple 234 — Une interface
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Une interface liste des méthodes qu'une classe doit fournir.
// ------------------------------------------------------------------
// Sortie attendue :
//   document
//   image
// ==================================================================

interface Affichable {
    func rendre() -> str;
}

class Document: Affichable {
    func rendre() -> str {
        return "document";
    }
}

class Image: Affichable {
    func rendre() -> str {
        return "image";
    }
}

let elements = [new Document(), new Image()];
for e in elements {
    println(e.rendre());
}
