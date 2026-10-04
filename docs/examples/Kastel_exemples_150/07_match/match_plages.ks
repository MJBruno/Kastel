// ====================================================================
// Kastel — match avec des plages
// Notions : a..b (fin exclue) et a..=b (fin incluse)
// Résultat attendu :
//   5 -> enfant
//   15 -> adolescent
//   30 -> adulte
//   70 -> senior
// ====================================================================

func categorie(age) {
    match age {
        0..13 => { println("{} -> enfant", age); },
        13..18 => { println("{} -> adolescent", age); },
        18..=64 => { println("{} -> adulte", age); },
        _ => { println("{} -> senior", age); }
    }
}

categorie(5);
categorie(15);
categorie(30);
categorie(70);
