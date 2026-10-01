// ==================================================================
// Exemple 174 — Découper en paquets
// Catégorie : Listes
// ------------------------------------------------------------------
// Regrouper les éléments par paquets de 2 avec slice.
// ------------------------------------------------------------------
// Sortie attendue :
//   [[1, 2], [3, 4], [5]]
// ==================================================================

let v = [1, 2, 3, 4, 5];
let paquets = [];
let i = 0;
while i < v.size() {
    paquets.add(v.slice(i, min(i + 2, v.size())));
    i += 2;
}
println(paquets);
