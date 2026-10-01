// ==================================================================
// Exemple 106 — Retour anticipé
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// return interrompt la fonction immédiatement.
// ------------------------------------------------------------------
// Sortie attendue :
//   négatif
//   nul
//   positif
// ==================================================================

func signe(n: int) -> str {
    if n < 0 {
        return "négatif";
    }
    if n == 0 {
        return "nul";
    }
    return "positif";
}

println(signe(-4));
println(signe(0));
println(signe(9));
