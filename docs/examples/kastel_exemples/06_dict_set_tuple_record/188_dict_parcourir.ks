// ==================================================================
// Exemple 188 — Parcourir un dict
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// for ... in sur un dict donne les clés.
// ------------------------------------------------------------------
// Sortie attendue :
//   pain coûte 1
//   lait coûte 2
//   oeuf coûte 3
// ==================================================================

let prix = {"pain": 1, "lait": 2, "oeuf": 3};
for produit in prix {
    println("{} coûte {}", produit, prix[produit]);
}
