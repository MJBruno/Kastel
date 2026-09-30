// ==================================================================
// Exemple 098 — Vider une liste avec while
// Catégorie : Structures de contrôle
// ------------------------------------------------------------------
// pop() renvoie le dernier élément (ou None si la liste est vide).
// ------------------------------------------------------------------
// Sortie attendue :
//   3
//   2
//   1
// ==================================================================

let pile = [1, 2, 3];
while !pile.is_empty() {
    println(pile.pop());
}
