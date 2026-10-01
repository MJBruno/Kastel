// ==================================================================
// Exemple 131 — Récursion mutuelle
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// est_pair et est_impair s'appellent l'une l'autre.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   true
// ==================================================================

func est_pair(n: int) -> bool {
    if n == 0 {
        return true;
    }
    return est_impair(n - 1);
}

func est_impair(n: int) -> bool {
    if n == 0 {
        return false;
    }
    return est_pair(n - 1);
}

println(est_pair(10));
println(est_impair(7));
