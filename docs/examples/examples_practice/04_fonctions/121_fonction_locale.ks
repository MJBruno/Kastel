// ==================================================================
// Exemple 121 — Fonction locale
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Une fonction peut en déclarer d'autres dans son corps.
// ------------------------------------------------------------------
// Sortie attendue :
//   25
// ==================================================================

func hypotenuse_carree(a: int, b: int) -> int {
    func carre(x: int) -> int {
        return x * x;
    }
    return carre(a) + carre(b);
}

println(hypotenuse_carree(3, 4));
