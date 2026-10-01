// ==================================================================
// Exemple 171 — Assembler deux listes
// Catégorie : Listes
// ------------------------------------------------------------------
// Parcourir deux listes en parallèle avec un index.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ada a 36 ans
//   Alan a 41 ans
// ==================================================================

let noms = ["Ada", "Alan"];
let ages = [36, 41];
for i in range(noms.size()) {
    println("{} a {} ans", noms[i], ages[i]);
}
