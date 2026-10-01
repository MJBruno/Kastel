// ==================================================================
// Exemple 468 — Mastermind : évaluer une proposition
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : compter les bien placés et les mal placés.
// ------------------------------------------------------------------
// Sortie attendue :
//   bien placés : 1, mal placés : 2
// ==================================================================

func evaluer(secret: List<int>, essai: List<int>) -> Tuple<int, int> {
    let bien = 0;
    let restes_s = [];
    let restes_e = [];
    for i in range(secret.size()) {
        if secret[i] == essai[i] {
            bien += 1;
        } else {
            restes_s.add(secret[i]);
            restes_e.add(essai[i]);
        }
    }
    let mal = 0;
    for x in restes_e {
        if restes_s.contains(x) {
            restes_s.remove(x);
            mal += 1;
        }
    }
    return (bien, mal);
}

let r = evaluer([1, 2, 3, 4], [1, 3, 2, 5]);
println("bien placés : {}, mal placés : {}", r[0], r[1]);
