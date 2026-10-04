// ====================================================================
// Kastel — Option avec match
// Notions : patterns Some(x) et None
// Résultat attendu :
//   trouvé à l'index 1
//   absent
// ====================================================================

func trouver(valeurs, cible) {
    for i in range(valeurs.size()) {
        if valeurs[i] == cible {
            return Some(i);
        }
    }
    return None;
}

match trouver([5, 8, 13], 8) {
    Some(i) => { println("trouvé à l'index {}", i); },
    None => { println("absent"); }
}

match trouver([5, 8, 13], 99) {
    Some(i) => { println("trouvé à l'index {}", i); },
    None => { println("absent"); }
}
