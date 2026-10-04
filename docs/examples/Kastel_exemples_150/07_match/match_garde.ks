// ====================================================================
// Kastel — match avec garde
// Notions : pattern + if condition
// Résultat attendu :
//   -5 : négatif
//   0 : zéro
//   12 : pair
//   7 : impair
// ====================================================================

func decrire(n) {
    match n {
        0 => { println("0 : zéro"); },
        x if x < 0 => { println("{} : négatif", x); },
        x if x % 2 == 0 => { println("{} : pair", x); },
        x => { println("{} : impair", x); }
    }
}

decrire(-5);
decrire(0);
decrire(12);
decrire(7);
