// ==================================================================
// Exemple 656 — Publication / abonnement
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : un éditeur envoie chaque message à deux abonnés ayant chacun leur canal.
// ------------------------------------------------------------------
// Sortie attendue :
//   12
// ==================================================================

let c1 = channel<int>();
let c2 = channel<int>();

func abonne1() -> int {
    let s = 0;
    for i in range(3) { s += c1.recv(); }
    return s;
}

func abonne2() -> int {
    let s = 0;
    for i in range(3) { s += c2.recv(); }
    return s;
}

let a = spawn(abonne1);
let b = spawn(abonne2);
for v in [1, 2, 3] {
    c1.send(v);
    c2.send(v);
}
println(a.join() + b.join());
