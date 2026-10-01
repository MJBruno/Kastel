// ==================================================================
// Exemple 168 — Aplatir une liste de listes
// Catégorie : Listes
// ------------------------------------------------------------------
// Deux boucles for qui ajoutent chaque élément dans une liste plate.
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 2, 3, 4, 5, 6]
// ==================================================================

let m = [[1, 2], [3], [4, 5, 6]];
let plat = [];
for sous in m {
    for x in sous {
        plat.add(x);
    }
}
println(plat);
