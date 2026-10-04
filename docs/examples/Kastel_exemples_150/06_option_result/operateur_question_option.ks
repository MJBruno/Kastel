// ====================================================================
// Kastel — L'opérateur ? sur Option
// Notions : propagation de None
// Résultat attendu :
//   Some(12)
//   None
// ====================================================================

func premier_pair(valeurs) -> Option<int> {
    for v in valeurs {
        if v % 2 == 0 {
            return Some(v);
        }
    }
    return None;
}

func double_du_premier_pair(valeurs) -> Option<int> {
    let p = premier_pair(valeurs)?;
    return Some(p * 2);
}

println(double_du_premier_pair([3, 5, 6, 8]));
println(double_du_premier_pair([1, 3]));
