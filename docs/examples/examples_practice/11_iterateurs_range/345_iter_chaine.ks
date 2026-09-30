// ==================================================================
// Exemple 345 — Pipeline complet
// Catégorie : Itérateurs et range
// ------------------------------------------------------------------
// Carrés des nombres impairs, en gardant les trois premiers.
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 9, 25]
// ==================================================================

let r = list(range(1, 20))
    .iter()
    .filter(n => n % 2 == 1)
    .map(n => n * n)
    .take(3)
    .collect();
println(r);
