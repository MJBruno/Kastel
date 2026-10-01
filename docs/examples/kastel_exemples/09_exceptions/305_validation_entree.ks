// ==================================================================
// Exemple 305 — Valider avec throw
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Refuser une valeur invalide au plus tôt.
// ------------------------------------------------------------------
// Sortie attendue :
//   4.0
//   racine d'un négatif
// ==================================================================

func racine(n: float) -> float {
    if n < 0 {
        throw "racine d'un négatif";
    }
    return sqrt(n);
}

println(racine(16.0));
try {
    racine(-4.0);
} catch (e) {
    println(e);
}
