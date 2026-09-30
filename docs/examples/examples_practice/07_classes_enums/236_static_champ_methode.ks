// ==================================================================
// Exemple 236 — Membres static
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// static appartient à la classe elle-même, pas aux objets ; on y accède par NomClasse.membre.
// ------------------------------------------------------------------
// Sortie attendue :
//   3.14159
//   81
// ==================================================================

class Maths {
    static let PI: float = 3.14159;

    static func carre(x: int) -> int {
        return x * x;
    }
}

println(Maths.PI);
println(Maths.carre(9));
