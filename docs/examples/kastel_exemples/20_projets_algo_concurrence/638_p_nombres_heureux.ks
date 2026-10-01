// ==================================================================
// Exemple 638 — Nombres heureux
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : répéter « somme des carrés des chiffres » jusqu'à atteindre 1 (heureux) ou 4 (cycle).
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 7, 10, 13, 19, 23, 28, 31, 32, 44, 49]
// ==================================================================

func heureux(n: int) -> bool {
    while n != 1 && n != 4 {
        let s = 0;
        while n > 0 {
            let d = n % 10;
            s += d * d;
            n = idiv(n, 10);
        }
        n = s;
    }
    return n == 1;
}

let res = [];
for n in range(1, 50) {
    if heureux(n) { res.add(n); }
}
println(res);
