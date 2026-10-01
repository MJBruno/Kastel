// ==================================================================
// Exemple 641 — Factorielle et limite des entiers
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : 20! tient sur 64 bits, 21! déclenche IntegerOverflow.
// ------------------------------------------------------------------
// Sortie attendue :
//   2432902008176640000
//   19
//   IntegerOverflow
// ==================================================================

func fact(n: int) -> int {
    let r = 1;
    for i in range(2, n + 1) {
        r *= i;
    }
    return r;
}

println(fact(20));
println(str(fact(20)).size());
try {
    fact(21);
} catch (e: Err) {
    println(e.kind);
}
