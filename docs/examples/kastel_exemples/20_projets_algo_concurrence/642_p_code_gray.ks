// ==================================================================
// Exemple 642 — Code de Gray
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : une suite binaire où deux valeurs consécutives ne diffèrent que d'un bit.
// ------------------------------------------------------------------
// Sortie attendue :
//   [0, 1, 3, 2, 6, 7, 5, 4]
// ==================================================================

let res = [];
for i in range(8) {
    res.add(i ^ (i >> 1));
}
println(res);
