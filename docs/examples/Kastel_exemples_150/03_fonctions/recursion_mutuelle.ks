// ====================================================================
// Kastel — Récursivité mutuelle
// Notions : deux fonctions qui s'appellent
// Résultat attendu :
//   true
//   true
//   false
// ====================================================================

func est_pair(n) {
    if n == 0 {
        return true;
    }
    return est_impair(n - 1);
}

func est_impair(n) {
    if n == 0 {
        return false;
    }
    return est_pair(n - 1);
}

println(est_pair(10));
println(est_impair(7));
println(est_pair(5));
