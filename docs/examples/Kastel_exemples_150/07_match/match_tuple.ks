// ====================================================================
// Kastel — match sur un tuple
// Notions : patterns tuple avec littéraux et liaisons
// Résultat attendu :
//   origine
//   sur l'axe x (4)
//   sur l'axe y (9)
//   point (2, 3)
// ====================================================================

func position(p) {
    match p {
        (0, 0) => { println("origine"); },
        (x, 0) => { println("sur l'axe x ({})", x); },
        (0, y) => { println("sur l'axe y ({})", y); },
        (x, y) => { println("point ({}, {})", x, y); }
    }
}

position((0, 0));
position((4, 0));
position((0, 9));
position((2, 3));
