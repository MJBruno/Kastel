// ==================================================================
// Exemple 179 — Modifier une liste dans une fonction
// Catégorie : Listes
// ------------------------------------------------------------------
// Les fonctions reçoivent la liste elle-même, pas une copie.
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 2, 0]
// ==================================================================

func ajouter_zero(v: List<int>) {
    v.add(0);
}

let v = [1, 2];
ajouter_zero(v);
println(v);
