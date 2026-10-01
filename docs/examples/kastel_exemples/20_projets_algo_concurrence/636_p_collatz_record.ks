// ==================================================================
// Exemple 636 — Plus long vol de Collatz
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : quel départ inférieur à 30 donne la plus longue suite ?
// ------------------------------------------------------------------
// Sortie attendue :
//   27 : 111 étapes
// ==================================================================

func etapes(n: int) -> int {
    let e = 0;
    while n != 1 {
        n = n % 2 == 0 ? idiv(n, 2) : 3 * n + 1;
        e += 1;
    }
    return e;
}

let meilleur = 1;
let record = 0;
for n in range(1, 30) {
    let e = etapes(n);
    if e > record {
        record = e;
        meilleur = n;
    }
}
println("{} : {} étapes", meilleur, record);
