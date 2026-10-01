// ==================================================================
// Exemple 096 — Minimum d'une liste (à la main)
// Catégorie : Structures de contrôle
// ------------------------------------------------------------------
// Garder le plus petit vu jusqu'ici.
// ------------------------------------------------------------------
// Sortie attendue :
//   1
// ==================================================================

let valeurs = [7, 3, 9, 1, 5];
let petit = valeurs[0];
for v in valeurs {
    if v < petit {
        petit = v;
    }
}
println(petit);
