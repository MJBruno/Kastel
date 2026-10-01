// ==================================================================
// Exemple 154 — Les listes sont partagées
// Catégorie : Listes
// ------------------------------------------------------------------
// Affecter une liste à une autre variable ne la copie pas : les deux noms désignent la même liste.
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 2, 3, 4]
// ==================================================================

let a = [1, 2, 3];
let b = a;        // même liste
b.add(4);
println(a);       // a voit la modification
