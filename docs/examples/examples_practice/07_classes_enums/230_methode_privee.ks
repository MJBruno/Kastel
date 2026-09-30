// ==================================================================
// Exemple 230 — Méthode privée
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Une méthode private est un détail interne, appelable seulement depuis la classe.
// ------------------------------------------------------------------
// Sortie attendue :
//   20
// ==================================================================

class Calculatrice {
    private func double(x: int) -> int {
        return x * 2;
    }

    func quadruple(x: int) -> int {
        return self.double(self.double(x));
    }
}

println(new Calculatrice().quadruple(5));
