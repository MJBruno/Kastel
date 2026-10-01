// ==================================================================
// Exemple 474 — Poker : reconnaître une main
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : compter les valeurs identiques pour classer une main de 5 cartes.
// ------------------------------------------------------------------
// Sortie attendue :
//   full
//   double paire
//   carré
//   carte haute
// ==================================================================

func classer(main: List<int>) -> str {
    let compte = dict();
    for v in main {
        compte[str(v)] = compte.get_or(str(v), 0) + 1;
    }
    let paires = 0;
    let brelan = false;
    let carre = false;
    for k in compte {
        if compte[k] == 2 { paires += 1; }
        if compte[k] == 3 { brelan = true; }
        if compte[k] == 4 { carre = true; }
    }
    if carre { return "carré"; }
    if brelan && paires == 1 { return "full"; }
    if brelan { return "brelan"; }
    if paires == 2 { return "double paire"; }
    if paires == 1 { return "paire"; }
    return "carte haute";
}

println(classer([5, 5, 5, 9, 9]));
println(classer([2, 2, 7, 7, 1]));
println(classer([8, 8, 8, 8, 3]));
println(classer([1, 4, 6, 9, 12]));
