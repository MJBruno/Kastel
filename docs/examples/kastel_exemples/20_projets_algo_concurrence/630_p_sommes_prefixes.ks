// ==================================================================
// Exemple 630 — Sommes préfixes
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : répondre à des questions « somme de i à j » en temps constant.
// ------------------------------------------------------------------
// Sortie attendue :
//   18
//   30
// ==================================================================

let v = [2, 4, 6, 8, 10];
let pref = [0];
for x in v {
    pref.add(pref[pref.size() - 1] + x);
}

func somme(i: int, j: int) -> int {
    return pref[j + 1] - pref[i];
}

println(somme(1, 3));
println(somme(0, 4));
