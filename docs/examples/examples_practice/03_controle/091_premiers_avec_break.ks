// ==================================================================
// Exemple 091 — Nombre premier avec break
// Catégorie : Structures de contrôle
// ------------------------------------------------------------------
// Sortir dès qu'un diviseur est trouvé.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

func est_premier(n: int) -> bool {
    if n < 2 {
        return false;
    }
    for d in range(2, n) {
        if d * d > n {
            break;
        }
        if n % d == 0 {
            return false;
        }
    }
    return true;
}

println(est_premier(17));
println(est_premier(18));
