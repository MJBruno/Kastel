// ====================================================================
// Kastel — match sur une liste
// Notions : [], [x], [a, b], [premier, ..]
// Résultat attendu :
//   liste vide
//   un seul élément : 7
//   deux éléments : 1 et 2
//   commence par 5
// ====================================================================

func analyser(valeurs) {
    match valeurs {
        [] => { println("liste vide"); },
        [x] => { println("un seul élément : {}", x); },
        [a, b] => { println("deux éléments : {} et {}", a, b); },
        [premier, ..] => { println("commence par {}", premier); }
    }
}

analyser([]);
analyser([7]);
analyser([1, 2]);
analyser([5, 6, 7, 8]);
