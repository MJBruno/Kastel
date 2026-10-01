// ==================================================================
// Exemple 487 — Tours de Hanoï : lister les déplacements
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : afficher la suite exacte des mouvements pour 3 disques.
// ------------------------------------------------------------------
// Sortie attendue :
//   7
//   A->C, A->B, C->B, A->C, B->A, B->C, A->C
// ==================================================================

let coups = [];

func hanoi(n: int, de: str, vers: str, via: str) {
    if n == 0 { return; }
    hanoi(n - 1, de, via, vers);
    coups.add(de + "->" + vers);
    hanoi(n - 1, via, vers, de);
}

hanoi(3, "A", "C", "B");
println(coups.size());
println(", ".join(coups));
