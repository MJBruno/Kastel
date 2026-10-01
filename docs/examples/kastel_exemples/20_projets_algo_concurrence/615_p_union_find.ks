// ==================================================================
// Exemple 615 — Union-Find (ensembles disjoints)
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : regrouper des éléments et tester s'ils sont connectés.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
//   3
// ==================================================================

let parent = [];
for i in range(7) { parent.add(i); }

func trouver(x: int) -> int {
    while parent[x] != x {
        parent[x] = parent[parent[x]];    // compression de chemin
        x = parent[x];
    }
    return x;
}

func unir(a: int, b: int) {
    parent[trouver(a)] = trouver(b);
}

unir(0, 1);
unir(2, 3);
unir(1, 3);
unir(4, 5);

println(trouver(0) == trouver(3));
println(trouver(0) == trouver(4));

let racines = Set();
for i in range(7) { racines.add(trouver(i)); }
println(racines.size());
