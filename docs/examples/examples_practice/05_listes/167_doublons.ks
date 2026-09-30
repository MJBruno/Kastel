// ==================================================================
// Exemple 167 — Supprimer les doublons
// Catégorie : Listes
// ------------------------------------------------------------------
// Ajouter à un résultat seulement si l'élément n'y est pas déjà.
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 2, 3, 4]
// ==================================================================

let v = [1, 2, 2, 3, 1, 4, 3];
let uniques = [];
for x in v {
    if !uniques.contains(x) {
        uniques.add(x);
    }
}
println(uniques);
