// ====================================================================
// Kastel — Membres statiques
// Notions : static let, static func, appel par NomClasse.membre
// Résultat attendu :
//   Kastel 1.0
//   16
//   10
// ====================================================================

class Outils {
    static let version: str = "1.0";

    static func nom() -> str {
        return "Kastel";
    }

    static func carre(n: int) -> int {
        return n * n;
    }

    static func somme(a: int, b: int) -> int {
        return a + b;
    }
}

println("{} {}", Outils.nom(), Outils.version);
println(Outils.carre(4));
println(Outils.somme(4, 6));
