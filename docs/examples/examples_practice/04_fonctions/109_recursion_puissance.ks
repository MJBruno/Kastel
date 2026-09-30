// ==================================================================
// Exemple 109 — Récursion : puissance rapide
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Exponentiation par carrés : on divise l'exposant par deux.
// ------------------------------------------------------------------
// Sortie attendue :
//   1024
//   243
// ==================================================================

func puissance(base: int, exp: int) -> int {
    if exp == 0 {
        return 1;
    }
    let moitie = puissance(base, idiv(exp, 2));
    if exp % 2 == 0 {
        return moitie * moitie;
    }
    return moitie * moitie * base;
}

println(puissance(2, 10));
println(puissance(3, 5));
